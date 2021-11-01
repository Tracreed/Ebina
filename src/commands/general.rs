use std::env;

use crate::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{error, info};
use std::os::unix::net::UnixStream;
use std::io::Write;
use std::io::prelude::*;
use std::io::BufReader;
use serde_json::{Value};
use humantime::format_duration;
use chrono::Duration;

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

#[command]
pub async fn mpv(ctx: &Context, msg: &Message) -> CommandResult {
	let pos = get_mpv_property("time-pos").unwrap().replace("\"", "").parse::<f64>().unwrap();
	let title = get_mpv_property("media-title").unwrap();

	&msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title(title);
			let dur = Duration::seconds(pos as i64);
			e.field("Position", format_duration(dur.to_std().unwrap()).to_string(), true);
			e
		});
		m
	}).await.unwrap();

	info!("{}", pos);
	Ok(())
}

fn get_mpv_property(property: &str) -> Result<String, &'static str> {
	let mut socket = match UnixStream::connect("/home/trac/.config/mpv/socket") {
		Ok(sock) => sock,
		Err(e) => {
			error!("Couldn't connect: {:?}", e);
			return Err("Couldn't connect to socket");
		}
	};
	let string_buf = format!("{{ \"command\": [\"get_property_string\", \"{}\"] }}\n", property);
	socket.write_all(string_buf.as_bytes()).unwrap();
    let mut response = String::new();
	let mut reader = BufReader::new(socket);
	reader.read_line(&mut response).unwrap();
	let v: Value = serde_json::from_str(&response).unwrap();
	Ok(v["data"].to_string())
}