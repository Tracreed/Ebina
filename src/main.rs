//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
mod commands;

pub mod models;
pub mod schema;

use std::{collections::HashSet, env, sync::Arc};

#[macro_use]
extern crate diesel;
extern crate bigdecimal;
extern crate dotenv;
extern crate osu_v2;
extern crate reqwest;
extern crate roxmltree;

use diesel::{pg::PgConnection, prelude::*};
use serenity::{
	async_trait,
	client::bridge::gateway::ShardManager,
	framework::{
		standard::macros::{group, help},
		standard::{
			help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
		},
	},
	http::Http,
	model::prelude::*,
	prelude::*,
};

use std::collections::HashMap;
use std::fs;

use serde_json;

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use self::models::{Charade, NewCharade};

use commands::{charades::*, general::*, osu::*, owner::*, vndb::*, moderation::*};

use bigdecimal::BigDecimal;

use url::Url;


pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}

pub struct OsuClientContainer;

impl TypeMapKey for OsuClientContainer {
	type Value = Mutex<osu_v2::client::Client>;
}

pub struct TagsContainer;

impl TypeMapKey for TagsContainer {
	type Value = HashMap<u64, commands::vndb::VnTagJ>;
}

#[help]
async fn my_help(
	context: &Context,
	msg: &Message,
	args: Args,
	help_options: &'static HelpOptions,
	groups: &[&'static CommandGroup],
	owners: HashSet<UserId>,
) -> CommandResult {
	let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await;
	Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		info!("Connected as {}", ready.user.name);
		info!("Guilds in cache: {}", ctx.cache.guilds().await.len());
	}

	async fn resume(&self, _: Context, _: ResumedEvent) {
		info!("Resumed");
	}
}

#[group]
#[commands(ping, quit, vn, invite)]
struct General;

#[group]
#[commands(play, add)]
struct Charades;

#[group]
#[commands(user)]
#[prefix("osu")]
#[default_command(user)]
#[description = "Commands related to the Osu! Rhythm game."]
struct Osu;

#[group]
#[commands(ban, kick, userinfo, guildinfo)]
#[description = "Commands related to moderation"]
struct Moderation;

#[tokio::main]
async fn main() {
	// This will load the environment variables located at `./.env`, relative to
	// the CWD. See `./.env.example` for an example on how to structure this.
	dotenv::dotenv().expect("Failed to load .env file");

	// Initialize the logger to use environment variables.
	//
	// In this case, a good default is setting the environment variable
	// `RUST_LOG` to debug`.
	let subscriber = FmtSubscriber::builder()
		.with_env_filter(EnvFilter::from_default_env())
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

	let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

	let http = Http::new_with_token(&token);

	// We will fetch your bot's owners and id
	let (owners, bot_id) = match http.get_current_application_info().await {
		Ok(info) => {
			let mut owners = HashSet::new();
			owners.insert(info.owner.id);

			(owners, info.id)
		}
		Err(why) => panic!("Could not access application info: {:?}", why),
	};

	// Create the framework
	let framework = StandardFramework::new()
		.configure(|c| {
			c.owners(owners)
				.prefixes(vec!["~", "ebina "])
				.on_mention(Some(bot_id))
		})
		.help(&MY_HELP)
		.group(&GENERAL_GROUP)
		.group(&CHARADES_GROUP)
		.group(&OSU_GROUP)
		.group(&MODERATION_GROUP);

	let mut client = Client::builder(&token)
		.framework(framework)
		.event_handler(Handler)
		.await
		.expect("Err creating client");

	{
		let mut data = client.data.write().await;
		data.insert::<ShardManagerContainer>(client.shard_manager.clone());
		let client_id = env::var("OSU_ID").expect("OSU_ID needs to be set");
		let client_secret = env::var("OSU_SECRET").expect("OSU_SECRET needs to be set");

		let osuclient = Mutex::new(osu_v2::client::Client::new(client_id, client_secret).await.expect("err creating osu client"));
		data.insert::<OsuClientContainer>(osuclient);
		data.insert::<TagsContainer>(HashMap::default());
	}

	parse_tags(&client, &std::path::Path::new("./vndb-tags-2021-02-08.json")).await;

	let shard_manager = client.shard_manager.clone();

	tokio::spawn(async move {
		tokio::signal::ctrl_c()
			.await
			.expect("Could not register ctrl+c handler");
		shard_manager.lock().await.shutdown_all().await;
	});

	let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));

	tokio::spawn(async move {
		loop {
			interval.tick().await;
			mangadex_update_xml(token.clone()).await;
		}
	});

	if let Err(why) = client.start_autosharded().await {
		error!("Client error: {:?}", why);
	}
}

pub fn establish_connection() -> PgConnection {
	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

use schema::*;

pub fn create_charade<'a>(
	conn: &PgConnection,
	category: &'a Categories,
	puzzle: &'a str,
	hint: &'a str,
	solution: &'a str,
	difficulty: &'a Difficulties,
	userid: &'a BigDecimal,
	public: &'a bool,
) -> Charade {
	let new_charade = NewCharade {
		category,
		hint,
		puzzle,
		solution,
		difficulty,
		userid,
		public,
	};

	diesel::insert_into(charades::table)
		.values(&new_charade)
		.get_result(conn)
		.expect("Error saving new post")
}

async fn parse_tags(client: &Client, path: &std::path::Path) {
	let mut data = client.data.write().await;
	let file_data = fs::read_to_string(path).expect("unable to read json file");
	let tags: Vec<commands::vndb::VnTagJ> = serde_json::from_str(&file_data).expect("unable to parse json");
	let tags_data = data.get_mut::<TagsContainer>().unwrap();
	for tag in tags {
		tags_data.insert(tag.id, tag);
	}
}

async fn mangadex_update_xml(token: String) {
	let http = Http::new_with_token(&token);
	let chan = http.get_channel(698062395263942689).await.unwrap();
	let chan_id = chan.id();
	let response = reqwest::get("https://mangadex.org/rss/DBqT28tXwbYdhnHNgA7QZceavSFrERk3/group_id/170?h=0").await.unwrap();
	let text = response.text().await.unwrap();
	let doc = roxmltree::Document::parse(&text).unwrap();
	let mut node = doc.descendants().find(|n| n.tag_name().name() == "item").unwrap()
	.first_element_child().unwrap();
	let title = node.text().unwrap();
	node = node.next_sibling_element().unwrap();
	let link = node.text().unwrap();
	node = node.next_sibling_element().unwrap();
	let mangalink = Url::parse(node.text().unwrap()).unwrap();
	let manga_id = mangalink.path_segments().unwrap().last().unwrap().parse::<u64>().unwrap();
	node = node.next_sibling_element().unwrap();
	let pubdate = chrono::DateTime::parse_from_str(node.text().unwrap(), "%a, %d %b %Y %H:%M:%S%:z").unwrap();
	if chrono::offset::Local::now().signed_duration_since(pubdate).num_minutes() < 5 {
		chan_id.send_message(&http, |m| {
			m.embed(|e| {
				e.title(title);
				e.url(link);
				e.thumbnail(format!("https://mangadex.org/images/manga/{}.large.jpg", manga_id));
				e
			});
			m
		})
		.await.unwrap();
	}
	println!("Title: {}, Link: {}, MangaLink: {}, Pub Date: {} cover: https://mangadex.org/images/manga/{}.large.jpg", title, link, mangalink, chrono::offset::Local::now().signed_duration_since(pubdate), manga_id);
}