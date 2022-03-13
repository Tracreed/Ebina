use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::utils::Colour;
use serenity::builder::CreateEmbedAuthor;

use url::Url;

use helium_api::*;

#[command]
pub async fn hotspot(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let hnt = args.rest();

	let mut address = true;

	if hnt.contains('-') {
		address = false;
	}

	println!("{}", hnt);

	let client = helium_api::Client::new_with_base_url("https://api.helium.io/v1".to_string(), "Ebina-bot");

	let htsp;

	if address {
		let hot = hotspots::get_address(&client, hnt).await;

		htsp = match hot {
			Ok(v) => v,
			Err(e) => {
				println!("{}", e);
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| {
						e.description("Could not find the hotspot!");
						e
					});
					m
				}).await?;
				return Ok(())
			},
		};
	} else {
		let hot = hotspots::get_name(&client, &hnt.to_lowercase()).await;

		htsp = match hot {
			Ok(v) => v[0].clone(),
			Err(e) => {
				println!("{}", e);
				msg.channel_id.send_message(&ctx.http, |m| {
					m.embed(|e| {
						e.description("Could not find the hotspot!");
						e
					});
					m
				}).await?;
				return Ok(())
			},
		};
	}

	println!("{:#?}", htsp);

	msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.field("Name", htsp.name.unwrap(), true);
			e.field("Gain", format!("{} dBi", htsp.gain.unwrap()), true);
			e.field("Elevation", format!("{} m", htsp.elevation.unwrap()), true);
			e
		})
	}).await?;
	Ok(())
}

pub async fn manage_helium_url(ctx: &Context, msg: &Message, url: Url) {

}