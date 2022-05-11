use std::env;

use crate::ShardManagerContainer;

use chrono::Utc;

use ebina_macro::tracking;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::diesel::prelude::*;
use crate::establish_connection;
use crate::models::*;
use crate::schema::*;

//use tracing::{error, info};

use wolfram_alpha::query::query;

use rustnao::{HandlerBuilder, Sauce};

extern crate openweather;

use openweather::{LocationSpecifier, Settings};

#[tracking("ping")]
#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    let latency = match runner.latency {
        Some(latency) => latency,
        None => {
            msg.reply(ctx, "Couldn't get latency. Maybe wait a little longer")
                .await?;

            return Ok(());
        }
    };

    let time_to_respond = Utc::now().signed_duration_since(msg.timestamp.with_timezone(&Utc));

    let latency_message = MessageBuilder::new()
        .push_line(format!(
            "Latency shard {}: {}ms",
            ctx.shard_id,
            latency.as_millis()
        ))
        .push_line(format!(
            "Time since message sent: {}ms",
            time_to_respond.num_milliseconds()
        ))
        .build();

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.field("Pong!", latency_message, true))
        })
        .await?;

    Ok(())
}

#[tracking("invite")]
#[command]
#[description = "Invite the bot to your server"]
pub async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let user = ctx.http.get_current_user().await?;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Invite me");
                e.url(format!(
                    "https://discord.com/oauth2/authorize?client_id={}&permissions=8198&scope=bot",
                    user.id
                ))
            });

            m
        })
        .await?;
    Ok(())
}

#[tracking("weather")]
#[command]
#[aliases("w")]
#[description = "Get current weather from city name"]
#[min_args(1)]
#[usage = "<city>"]
#[example = "London"]
pub async fn weather(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let key = env::var("WEATHER_KEY").expect("WEATHER_KEY needs to be set");
    let mut city = Vec::new();
    for arg in args.iter::<String>() {
        city.push(arg.unwrap());
    }
    let location = LocationSpecifier::CityAndCountryName {
        city: city.join(" "),
        country: "".to_string(),
    };
    let settings = Settings {
        unit: Some(openweather::Unit::Metric),
        lang: Some(openweather::Language::English),
    };
    let w = openweather::get_current_weather(&location, &key, &settings).unwrap();
    println!("{:?}", w);
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} - {}", w.name, w.weather[0].description));
                e.thumbnail(format!(
                    "https://openweathermap.org/img/wn/{}@2x.png",
                    w.weather[0].icon
                ));
                e.field(
                    "Temperature",
                    format!(
                        "{}°, Feels like: {}°",
                        w.main.temp as i32, w.main.feels_like as i32
                    ),
                    true,
                );
                e.field("Humidity", format!("{}%", w.main.humidity), true);
                e
            });
            m
        })
        .await
        .unwrap();
    Ok(())
}

#[tracking("wolfram")]
#[command]
#[aliases("s")]
#[description = "Ask WolframAlpha questions about anything"]
pub async fn wolf(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let app_id = env::var("WOLFRAM_ALPHA").expect("WOLFRAM_ALPHA needs to be set");
    let response = query(None, &app_id, args.rest(), None).unwrap();
    println!("{:?}", response);

    if response.pods.is_none() {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.add_embed(|e| {
                    e.title("WolframAlpha");
                    if response.parsetiming.eq(&0.0) {
                        e.field("Reason", "Ratelimited!", false);
                    }
                    e.description("Failed!");
                    e.color(serenity::utils::Color::from_rgb(221, 17, 0));
                    e
                });
                m
            })
            .await?;
        return Ok(());
    };

    let pods = response.pods.unwrap();
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.add_embed(|e| {
                e.title("WolframAlpha");
                e.field(
                    "Interpretation",
                    pods[0].subpods[0].plaintext.as_ref().unwrap(),
                    false,
                );
                e.field(
                    "Result",
                    pods[1].subpods[0].plaintext.as_ref().unwrap(),
                    false,
                );
                e.color(serenity::utils::Color::from_rgb(221, 17, 0));
                e
            });
            m
        })
        .await?;
    Ok(())
}

#[tracking("sauce")]
#[command]
#[usage = "<image_link> or <attached image>"]
#[description = "Get the sauce of an image from SauceNAO"]
pub async fn sauce(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let api_key = env::var("SAUCENAO").expect("SAUCENAO needs to be set");
	let url_arg = args.single::<String>();
	
	let url = if !msg.attachments.is_empty() {
		msg.attachments[0].url.clone()
	} else {
		match url_arg {
			Ok(u) => u,
			Err(_) => {
				msg.channel_id
					.send_message(&ctx.http, |m| {
						m.add_embed(|e| {
							e.title("Error!");
							e.description("Command requires an attached image or url.");
							e
						});
						m
					})
					.await?;
					return Ok(())
			}
		}
	};

	let handler = HandlerBuilder::default().api_key(&api_key).num_results(1).build();

	handler.set_min_similarity(45);

	let result: Vec<Sauce> = handler.get_sauce(&url, None, None).unwrap();

	if result.is_empty() {
		msg.channel_id
			.send_message(&ctx.http, |m| {
				m.add_embed(|e| {
					e.title("Error!");
					e.description("No match found!");
					e
				});
				m
			})
			.await?;
		return Ok(());
	}

	let sauce = &result[0];

	msg.channel_id
        .send_message(&ctx.http, |m| {
            m.add_embed(|e| {
                e.title("Match Found");
                e.thumbnail(sauce.thumbnail.clone());
				match &sauce.additional_fields {
					Some(v) => {
						let source = v.get("source");
						if let Some(v) = source {
							let s = v.as_str().unwrap();
							if !s.is_empty() {
								e.field("Source", s, false);
							}
						}
					},
					None => {},
				};
				e.field("Site", sauce.site.clone(), false);
				if sauce.source.is_some() {
					e.field("Source", sauce.source.as_ref().unwrap().clone(), false);
				}

				if sauce.creator.is_some() && !sauce.creator.as_ref().unwrap().is_empty() {
					e.field("Creator", sauce.creator.as_ref().unwrap().join("\n"), false);
				}
				e.field("Similarity", format!("{}%", sauce.similarity), false);
				if !sauce.ext_urls.is_empty() {
					e.field("External URLs", sauce.ext_urls[0].clone(), false);
				}
                e
            });
            m
        })
        .await.unwrap();

	Ok(())
}

#[tracking("prefix")]
#[command]
#[required_permissions("MANAGE_GUILD")]
pub async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let connection = establish_connection();

    let prefix = args.rest();

    create_server_settings(&connection, &(msg.guild_id.unwrap().0 as i64), prefix);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.add_embed(|e| {
                e.title("Set prefix to");
                e.description(prefix);
                e
            });
            m
        })
        .await?;

    Ok(())
}

pub fn create_server_settings<'a>(
    conn: &PgConnection,
    server: &'a i64,
    prefix: &str,
) -> ServerSettings {
    let new_server_setting = NewServerSettings {
        server_id: server,
        prefix: &prefix.to_string(),
    };

    use crate::schema::discord_settings::server_id;

    diesel::insert_into(discord_settings::table)
        .values(&new_server_setting)
        .on_conflict(server_id)
        .do_update()
        .set(discord_settings::prefix.eq(prefix))
        .get_result(conn)
        .expect("Error saving new guild setting")
}
