use serenity::{framework::standard::{macros::command, Args, CommandResult}};
use serenity::model::prelude::*;
use serenity::http::Http;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::utils::MessageBuilder;
use crate::ConnectionContainer;
use mangadex_api::MangaDexClient;
use ron;
use uuid::Uuid;

use crate::models::{self, Feed};

use std::collections::HashMap;

use tracing::{info};

#[derive(Clone, Debug)]
struct FeedGroup {
	manga_id: u64,
	title: String,
	last: f64,
	last_id: u64,
	first: f64,
	first_id: u64,
}

trait Update {
	fn set_last(&mut self, last: f64, last_id: u64);
	fn set_first(&mut self, first: f64, first_id: u64);
}

impl Update for FeedGroup {
	fn set_last(&mut self, last: f64, last_id: u64) {
		self.last = last;
		self.last_id = last_id;
	}

	fn set_first(&mut self, first: f64, first_id: u64) {
		self.first = first;
		self.first_id = first_id;
	}
}

#[command]
#[owners_only]
#[min_args(1)]
pub async fn set(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let client = MangaDexClient::default();
	let guild_id = msg.guild_id.unwrap();
	let channel_id = msg.channel_id;
	let group_id = Uuid::parse_str(args.single::<String>().unwrap().as_str()).unwrap();
	let group_res = client.scanlation_group()
		.get()
		.group_id(&group_id)
		.build()?
		.send()
		.await?;
	let group = group_res.data;

    let pool = {
        let data = ctx.data.read().await;
        data.get::<ConnectionContainer>().unwrap().clone()
    };

    sqlx::query("INSERT INTO feeds (server_id, channel_id, manga_id) VALUES ($1, $2, $3)")
        .bind(guild_id.0 as i64)
        .bind(channel_id.0 as i64)
        .bind(group_id.to_string())
        .execute(&pool)
        .await?;

	let group_name = group.attributes.name.clone();
	msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title("Ebina");
			let message = MessageBuilder::new()
				.push("Set ")
				.mention(&channel_id)
				.push(" as announcement channel for ")
				.push(group_name)
				.build();
			e.description(message);
			e
		});
		m
	})
	.await.unwrap();
	println!("{:?}", group.attributes.name);
	Ok(())
}

#[command]
#[owners_only]
pub async fn unset(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionContainer>().unwrap();
    let channel_id = msg.channel_id.0 as i64;

    sqlx::query("DELETE FROM feeds WHERE channel_id = $1")
        .bind(channel_id)
        .execute(pool)
        .await?;

	Ok(())
}

#[command]
#[owners_only]
pub async fn role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let manga_id = args.single::<u64>()?;
	let guild_id = msg.guild_id.unwrap().0 as i64;
	let role_id: u64;
    let data = ctx.data.read().await;
    let pool = data.get::<ConnectionContainer>().unwrap();

	if msg.mention_roles.len() > 0 {
		role_id = msg.mention_roles[0].0;
	} else {
		return Ok(());
	}

	let mut map: HashMap<u64, u64>;

    let role: Result<Option<models::Role>, sqlx::Error> = sqlx::query_as("SELECT * FROM roles WHERE server_id = $1")
        .bind(guild_id)
        .fetch_optional(pool)
        .await;

	if let Ok(Some(role)) = role {
		map = ron::from_str(&role.data).unwrap_or(HashMap::new());
	} else {
		map = HashMap::new();
	}

	map.insert(manga_id, role_id);

    sqlx::query("INSERT INTO roles (server_id, data) VALUES ($1, $2) ON CONFLICT (server_id) DO UPDATE SET data = $2")
        .bind(guild_id)
        .bind(ron::to_string(&map).unwrap())
        .execute(pool)
        .await?;

	Ok(())
}


pub async fn check_feeds(token: String, pool: &sqlx::PgPool) {
	let http = Http::new(&token);

	let md_client = MangaDexClient::default();

    let feeds: Vec<Feed> = sqlx::query_as("SELECT * FROM feeds")
        .fetch_all(pool)
        .await
        .expect("Error loading posts");

	for feed in feeds {
		let group_res = md_client.scanlation_group().get().group_id(&Uuid::parse_str(&feed.manga_id).unwrap()).build().unwrap().send().await.unwrap();
		let relationships = group_res.data.relationships;

        let role: Result<Option<models::Role>, sqlx::Error> = sqlx::query_as("SELECT * FROM roles WHERE server_id = $1")
            .bind(feed.server_id)
            .fetch_optional(pool)
            .await;

		let role_data: HashMap<u64, u64>;

		if let Ok(Some(role)) = role {
			role_data = ron::from_str(&role.data).unwrap();
		} else {
			role_data = HashMap::new();
		}

		let mut chapters_group = HashMap::new();
        for relationship in &relationships {
            if relationship.type_ == mangadex_api_types::RelationshipType::Chapter {
                let chapter_res = md_client.chapter().get().chapter_id(&relationship.id).build().unwrap().send().await.unwrap();
                let chapter_attributes = chapter_res.data.attributes;
                let manga_id = chapter_res.data.relationships.iter().find(|r| r.type_ == mangadex_api_types::RelationshipType::Manga).unwrap().id;

                let publish_at: chrono::DateTime<chrono::offset::Utc> = chapter_attributes.publish_at.to_string().parse().unwrap();
                if chrono::offset::Local::now().signed_duration_since(publish_at).num_minutes() > 10 {
                    break;
                }

                let mut groups_vec = Vec::<String>::new();
                for group_rel in &relationships {
                    if group_rel.type_ == mangadex_api_types::RelationshipType::ScanlationGroup {
                        if let Some(mangadex_api_schema::v5::RelatedAttributes::ScanlationGroup(group_attributes)) = group_rel.attributes.as_ref() {
                            groups_vec.push(group_attributes.name.clone());
                        }
                    }
                }

                let feed_group = FeedGroup {
                    manga_id: manga_id.as_u64_pair().0,
                    title: chapter_attributes.title.clone(),
                    last: chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap(),
                    last_id: relationship.id.as_u64_pair().0,
                    first: chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap(),
                    first_id: relationship.id.as_u64_pair().0,
                };
                info!("{:?}", feed_group);
                let chapter_group = match chapters_group.get_mut(&manga_id.as_u64_pair().0) {
                    Some(v) => {v},
                    None => {
                        chapters_group.insert(manga_id.as_u64_pair().0, feed_group.clone());
                        continue;
                    },
                };

                if chapter_group.last < chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap() {
                    chapter_group.set_last(chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap(), relationship.id.as_u64_pair().0);
                }

                if chapter_group.first > chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap() {
                    chapter_group.set_first(chapter_attributes.chapter.clone().unwrap_or("0".to_string()).parse::<f64>().unwrap(), relationship.id.as_u64_pair().0);
                }
            }
        }
		let channel = http.get_channel(feed.channel_id as u64).await.unwrap().guild().unwrap();

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
					e.thumbnail(format!("https://mangadex.org/images/manga/{}.jpg", chapter.manga_id));
					e.color(Colour::from_rgb(246, 131, 40));
					e
				});
				if role_data.contains_key(&chapter.manga_id) {
					let r = RoleId{0: *role_data.get(&chapter.manga_id).unwrap()};
					let message = MessageBuilder::new()
					.mention(&r)
					.build();
					m.content(message);
				}
				m
			})
			.await.unwrap();
		}
	}
}