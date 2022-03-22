mod commands;

pub mod models;
pub mod schema;
pub mod utils;

use std::{collections::HashSet, env, sync::{Arc, atomic::AtomicBool}};

#[macro_use]
extern crate diesel;
extern crate bigdecimal;
extern crate osu_v2;
extern crate reqwest;
extern crate roxmltree;

#[macro_use]
extern crate diesel_migrations;


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

use crate::models::*;

use tokio::signal::unix::{signal, SignalKind};

use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

use url::Url;

use commands::{
    anilist::*, charades::*, general::*, mangadex::*, moderation::*, osu::*, owner::*, vndb::*
};

embed_migrations!();

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionContainer;

impl TypeMapKey for ConnectionContainer {
    type Value = Mutex<PgConnection>;
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
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}


struct Handler {
	is_web_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        let guilds = ctx.cache.guilds().await.len();
        info!("Guilds in cache: {}", guilds);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

	async fn message(&self, ctx: Context, msg: Message) {

		let raw_msg = msg.content.clone();

		let words = raw_msg.split(' ');

		for word in words {
			let url_contains = Url::parse(word);
			match url_contains {
				Ok(url) => handle_url(&ctx, &msg, url).await,
				Err(_) => continue,
			}
		}
	}

	async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {

		info!("Cache ready!");

		if !self.is_web_running.load(std::sync::atomic::Ordering::Relaxed) {
			tokio::spawn(async move {
				let app = ebina_web::WebApp::new(ctx.clone());
				app.start("0.0.0.0:8081".to_string()).await.unwrap();
				info!("Started the web app");
			});
		}
		self.is_web_running.swap(true, std::sync::atomic::Ordering::Relaxed);
	}
}

/// Handle messages that are just URLs.
async fn handle_url(ctx: &Context, msg: &Message, url: Url) {
	let domain = match url.domain() {
		Some(v) => v,
		None => {
			error!("Something went wrong with the url: {}", url);
			return
		},
	};

	match domain {
		"mangadex.org" => {
			manage_md_url(ctx, msg, url).await;
		}
		"anilist.co" => {

		}
		_ => {},
	}
}

#[group]
#[commands(ping, quit, vn, invite, weather, wolf, sauce, prefix)]
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
#[commands(ban, kick, userinfo, guildinfo, avatar, clear)]
#[description = "Commands related to moderation"]
struct Moderation;

/* #[group]
#[prefix = "feed"]
#[commands(set, unset, role)]
struct Feed; */

#[group]
#[commands(manga)]
//#[sub_groups(feed)]
#[default_command(manga)]
#[prefix("md")]
#[description = "Commands related to MangaDex"]
struct Mangadex;

#[group]
#[commands(anilist_search, anilist_manga, anilist_anime, anilist_schedule)]
#[default_command(anilist_search)]
#[prefix("al")]
#[description = "Commands related to Anilist and Anichart"]
struct AniList;

#[tokio::main]
async fn main() {
    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let prefix = env::var("PREFIX").expect("Expected a prefix in the environment");

    let http = Http::new_with_token(&token);

	let connection = establish_connection();

	embedded_migrations::run(&connection).unwrap();

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
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move {
                        use crate::schema::discord_settings::dsl::*;

                        let guild = msg.guild_id.unwrap().0;

                        let data = ctx.data.read().await;

                        let conn = &*data.get::<ConnectionContainer>().unwrap().lock().await;
                        let result = discord_settings
                            .filter(server_id.eq(guild as i64))
                            .limit(1)
                            .load::<ServerSettings>(conn);
                        match result {
                            Ok(v) => {
                                if v.is_empty() {
                                    None
                                } else {
                                    Some(v[0].prefix.clone())
                                }
                            }
                            Err(_) => None,
                        }
                    })
                })
                .prefixes(vec![prefix.as_str(), "ebina "])
                .on_mention(Some(bot_id))
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&CHARADES_GROUP)
        .group(&OSU_GROUP)
        .group(&MODERATION_GROUP)
        .group(&MANGADEX_GROUP)
        .group(&ANILIST_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler {
			is_web_running: AtomicBool::new(false),
		})
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        let client_id = env::var("OSU_ID").expect("OSU_ID needs to be set");
        let client_secret = env::var("OSU_SECRET").expect("OSU_SECRET needs to be set");

        let connection = Mutex::new(establish_connection());

        data.insert::<ConnectionContainer>(connection);

        let osuclient = Mutex::new(
            osu_v2::client::Client::new(client_id, client_secret)
                .await
                .expect("err creating osu client"),
        );
        data.insert::<OsuClientContainer>(osuclient);
        data.insert::<TagsContainer>(HashMap::default());
    }

    parse_tags(
        &client
    )
    .await;

    let shard_manager_ctrl = client.shard_manager.clone();
    let shard_manager_term = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager_ctrl.lock().await.shutdown_all().await;
    });

    let mut termsig = signal(SignalKind::terminate()).unwrap();

    tokio::spawn(async move {
        termsig.recv().await;
        shard_manager_term.lock().await.shutdown_all().await;
    });

    /* let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            //mangadex_update_xml(token.clone()).await;
            check_feeds(token.clone()).await;
        }
    }); */

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {}, because of {}", database_url, e))
}

async fn parse_tags(client: &Client) {
    let mut data = client.data.write().await;

    let file_data = include_str!("../assets/vndb-tags-2021-02-08.json");

    let tags: Vec<commands::vndb::VnTagJ> =
        serde_json::from_str(file_data).expect("unable to parse json");
    let tags_data = data.get_mut::<TagsContainer>().unwrap();

    for tag in tags {
        tags_data.insert(tag.id, tag);
    }
}
