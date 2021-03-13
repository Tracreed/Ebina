use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::utils::MessageBuilder;
use crate::{MDClientContainer};
use mangadex_api::v2::{responses::*};

use tracing::{info};

use regex::Regex;

#[command]
pub async fn manga(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let data = ctx.data.read().await;
	let client = data.get::<MDClientContainer>().unwrap().lock().await;
	let mut title = Vec::new();
	for arg in args.iter::<String>() {
		title.push(arg.unwrap());
	}
	let mut search = client.search_scrape();
	let search_res = search.title(title.join(" ")).send().await?;
	if search_res.len() == 0 {
		let message = MessageBuilder::new()
		.push("No manga named ")
		.push_mono_safe(&title.join(" "))
		.push(" found.")
		.build();
		&msg.channel_id.say(&ctx, message).await?;
		return Ok(())
	}
	let scrape_manga = &search_res[0];
	let manga_res = client.manga(*scrape_manga.manga_id()).send().await?.ok().unwrap();
	let manga = manga_res.data();
	info!("{:?}", manga);
	&msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			let formats: Vec<String> = manga.tags().iter().filter(|t| t.category() == mangadex_api::types::TagCategory::Format).map(|t| {
				t.to_string()
			}).collect();
			let genres: Vec<String> = manga.tags().iter().filter(|t| t.category() == mangadex_api::types::TagCategory::Genre).map(|t| {
				t.to_string()
			}).collect();
			let contents: Vec<String> = manga.tags().iter().filter(|t| t.category() == mangadex_api::types::TagCategory::Content).map(|t| {
				t.to_string()
			}).collect();
			e.title(manga.title());
			e.color(Colour::from_rgb(246, 131, 40));
			e.url(format!("https://mangadex.org/title/{}", manga.id()));
			e.thumbnail(manga.main_cover_url());
			e.description(fix_description(manga.description().to_string()));
			e.author(|a| {
				a.name("MangaDex");
				a.icon_url("https://i.imgur.com/gFzVv6g.png");
				a.url("https://mangadex.org/");
				a
			});
			if manga.authors().len() > 0 {
				e.field("Author", manga.authors().join(", "), true);
			}
			if manga.artists().len() > 0 {
				e.field("Artist", manga.artists().join(", "), true);
			}
			if manga.publication().demographic() != &mangadex_api::types::Demographic::None {
				e.field("Demographic", manga.publication().demographic(), true);
			}
			if contents.len() > 0 {
				e.field("Content", contents.join(", "), true);
			}
			if formats.len() > 0 {
				e.field("Format", formats.join(", "), true);
			}
			e.field("Rating", format!("Bayesian rating: {}\nMean rating: {}\nUsers: {}", manga.rating().bayesian(), manga.rating().mean(), manga.rating().users()), true);
			e.field("Pub. Status", manga.publication().status(), true);
			if genres.len() > 0 {
				e.field("Genre", genres.join(", "), true);
			}
			e
		});
		m
	})
	.await.unwrap();
	Ok(())
}

fn fix_description(mut description: String) -> String {
	let bold = Regex::new(r"\[(|/)b\]").unwrap();
	let spoilers = Regex::new(r"\[spoiler\].*\[/spoiler\]").unwrap();
	let language = Regex::new(r"(\[b\]\[u\]|\[u\]\[b\]).*(\[/u\]\[/b\]|\[/b\]\[/u\])(\n| |\r\n)\[spoiler\].*(\[/spoiler\]|)").unwrap();
	let horizontal = Regex::new(r"\[hr\](\n|)").unwrap();
	description = language.replace_all(&description, "").to_string();
	description = bold.replace_all(&description, "**").to_string();
	description = spoilers.replace_all(&description, "").to_string();
	description = horizontal.replace_all(&description, "").to_string();
	description = description.replace("&quot;", "\"");
	if description.len() > 1000 {
		description = format!("{}...", &description[..1000]);
	}
	description
}