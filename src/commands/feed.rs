use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::http::Http;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::utils::MessageBuilder;
use crate::{MDClientContainer, establish_connection};
use mangadex_api::v2::{responses::*, MangaDexV2};
use ron::*;

use crate::models::*;
use crate::schema::*;

use crate::diesel::prelude::*;

use std::collections::HashMap;

use tracing::{info};

#[derive(Clone, Debug)]
struct FeedGroup {
	manga_id: u64,
	title: String,
	last: u64,
	last_id: u64,
	first: u64,
	first_id: u64,
	group: Vec<String>,
	chapters: u64,
}

trait Update {
	fn set_last(&mut self, last: u64, last_id: u64);
	fn set_first(&mut self, first: u64, first_id: u64);
}

impl Update for FeedGroup {
	fn set_last(&mut self, last: u64, last_id: u64) {
		self.last = last;
		self.last_id = last_id;
	}

	fn set_first(&mut self, first: u64, first_id: u64) {
		self.first = first;
		self.first_id = first_id;
	}
}

#[command]
#[owners_only]
#[min_args(1)]
pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let data = ctx.data.read().await;
	let client = data.get::<MDClientContainer>().unwrap().lock().await;
	let guild_id = msg.guild_id.unwrap();
	let channel_id = msg.channel_id;
	let group_id = args.single::<i64>().unwrap();
	let group_res = client.group(group_id as u64).send().await?.ok().unwrap();
	let group = group_res.data();
	let conn = establish_connection();

	create_feed(&conn, &(guild_id.0 as i64), &(channel_id.0 as i64), &group_id);
	&msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title("Ebina");
			let message = MessageBuilder::new()
				.push("Set ")
				.mention(&channel_id)
				.push(" as announcement channel for ")
				.push(group.name())
				.build();
			e.description(message);
			e
		});
		m
	})
	.await.unwrap();
	println!("{:?}", group);
	Ok(())
}

#[command]
#[owners_only]
#[min_args(1)]
pub async fn unset(ctx: &Context, msg: &Message) -> CommandResult {
	let channel_id = msg.channel_id;
	let conn = establish_connection();
	delete_feed(&conn, &(channel_id.0 as i64));
	Ok(())
}


pub async fn check_feeds(token: String) {
	use crate::schema::feeds::dsl::*;
	use crate::schema::roles::dsl::*;

	let connection = establish_connection();

	let http = Http::new_with_token(&token);

	

	let md_client = MangaDexV2::default();

	let results = feeds
		.load::<Feed>(&connection)
		.expect("Error loading posts");
	for feed in results {
		let group_res = md_client.group_chapters(feed.manga as u64).limit(10).send().await.unwrap().ok().unwrap();
		let chapters = group_res.data().chapters();
		let groups = group_res.data().groups();
		let roles_res = roles
			.filter(crate::schema::roles::columns::server_id.eq(feed.server as i64))
			.load::<crate::models::Role>(&connection)
			.unwrap();
		let mut chapters_group = HashMap::new();
		for chapter in chapters {
			if chrono::offset::Local::now().signed_duration_since(*chapter.timestamp()).num_minutes() > 10 {
				break;
			}

			let mut groups_vec = Vec::<String>::new();
			for group in groups {
				groups_vec.push(group.name().clone());
			};

			let feed_group = FeedGroup {
			    manga_id: *chapter.manga_id(),
			    title: chapter.manga_title().to_string(),
			    last: chapter.chapter().parse::<u64>().unwrap(),
			    last_id: *chapter.id(),
			    first: chapter.chapter().parse::<u64>().unwrap(),
			    first_id: *chapter.id(),
			    group: groups_vec,
				chapters: 1,
			};
			info!("{:?}", feed_group);
			let chapter_group = match chapters_group.get_mut(chapter.manga_id()) {
				Some(v) => {v},
				None => {
					chapters_group.insert(chapter.manga_id(), feed_group.clone());
					continue;
				},
			};

			if chapter_group.last < chapter.chapter().parse::<u64>().unwrap() {
				chapter_group.set_last(chapter.chapter().parse::<u64>().unwrap(), *chapter.id());
			}

			if chapter_group.first > chapter.chapter().parse::<u64>().unwrap() {
				chapter_group.set_first(chapter.chapter().parse::<u64>().unwrap(), *chapter.id());
			}
		}
		let channel = http.get_channel(feed.channel as u64).await.unwrap().guild().unwrap();

		for (_, chapter) in chapters_group.iter() {
			channel.id.send_message(&http, |m| {
				m.embed(|e| {
					if chapter.last != chapter.first {
						e.title(format!("Ch. {} - {} - {}", chapter.first, chapter.last, chapter.title));
					} else {
						e.title(format!("Ch. {} - {}", chapter.last, chapter.title));
					}
					e.description(format!("New chapter is out! [Click here to read](https://mangadex.org/chapter/{})", chapter.first_id));
					e.author(|a| {
						a.name("MangaDex");
						a.icon_url("https://i.imgur.com/gFzVv6g.png");
						a.url("https://mangadex.org/");
						a
					});
					e.color(Colour::from_rgb(246, 131, 40));
					e.footer(|f| {
						f.text(chapter.group[0].clone());
						f
					});
					e
				});
				m
			})
			.await.unwrap();
		}
	}
}

pub fn create_feed<'a>(conn: &PgConnection, server: &'a i64, channel: &'a i64, manga: &'a i64) -> Feed {
	let new_feed = NewFeed {
		server_id: server,
		channel_id: channel,
		manga_id: manga
	};
	diesel::insert_into(feeds::table)
		.values(&new_feed)
		.get_result(conn)
		.expect("Error saving new feed")
}

pub fn delete_feed<'a>(conn: &PgConnection, channel: &'a i64) {
	use crate::schema::feeds::dsl::*;

	diesel::delete(feeds.filter(channel_id.eq(channel))).execute(conn).unwrap();
}