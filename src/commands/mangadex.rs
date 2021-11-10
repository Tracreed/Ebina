use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::utils::MessageBuilder;

use mangadex_api::MangaDexClient;
use mangadex_api::types::Language;
use mangadex_api::types::RelationshipType;
use mangadex_api::CDN_URL;



use tracing::{info};

use regex::Regex;

#[command]
pub async fn manga(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let client = MangaDexClient::default();

	let mut title = Vec::new();
	for arg in args.iter::<String>() {
		title.push(arg.unwrap());
	}

	let manga_res = client
		.manga()
		.search()
		.title(title.join(" "))
		.includes(vec!(RelationshipType::Author))
		.build()?
		.send()
		.await?;


	let manga = manga_res.data[0].clone();

	info!("{:?}", manga);

	let manga_title = manga.attributes.title.get_key_value(&Language::English).unwrap().1;
	let manga_description = manga.attributes.description.get_key_value(&Language::English).unwrap().1;
	let manga_cover_id = manga.relationships
		.iter()
		.find(|related| related.type_ == RelationshipType::CoverArt)
		.expect("no cover art found for manga")
		.id;
	let manga_cover = client
		.cover()
		.get()
		.cover_id(&manga_cover_id)
		.build()?
		.send()
		.await?;
	let manga_authors = manga.relationships
		.iter()
		.filter(|related| related.type_ == RelationshipType::Author)
		.collect::<Vec<_>>();

	let _ = &msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title(manga_title);
			e.color(Colour::from_rgb(246, 131, 40));
			e.url(format!("https://mangadex.org/title/{}", manga.id));
			e.thumbnail(&format!(
				"{}/covers/{}/{}",
				CDN_URL, manga.id, manga_cover.data.attributes.file_name
				)
			);
			e.description(fix_description(manga_description.to_string()));
			e.author(|a| {
				a.name("MangaDex");
				a.icon_url("https://i.imgur.com/gFzVv6g.png");
				a.url("https://mangadex.org/");
				a
			});

			e.field("Author", manga_authors.iter().map(|auth| {
				auth.id.to_hyphenated().to_string()
			}).collect::<Vec<_>>().join(" "), true);

			/*if manga.artists().len() > 0 {
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
			}*/
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