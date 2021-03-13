use std::env;

use crate::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;


extern crate openweather;

use openweather::{LocationSpecifier, Settings};

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
			&msg.reply(ctx, "Couldn't get latency. Maybe wait a little longer")
				.await?;

			return Ok(());
		}
	};

	msg.reply(&ctx.http, format!("Latency: {}ms", latency.as_millis()))
		.await?;
	Ok(())
}

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
	let location = LocationSpecifier::CityAndCountryName{city : city.join(" "), country : "".to_string()};
	let settings = Settings{
	    unit: Some(openweather::Unit::Metric),
	    lang: Some(openweather::Language::English),
		
	};
	let w = openweather::get_current_weather(&location, &key, &settings).unwrap();
	println!("{:?}", w);
	&msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title(format!("{} - {}", w.name, w.weather[0].description));
			e.thumbnail(format!("https://openweathermap.org/img/wn/{}@2x.png", w.weather[0].icon));
			e.field("Temperature", format!("{}°, Feels like: {}°", w.main.temp as i32, w.main.feels_like as i32), true);
			e.field("Humidity", format!("{}%", w.main.humidity), true);
			e
		});
		m
	})
	.await.unwrap();
	Ok(())
}