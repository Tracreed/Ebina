use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::builder::{CreateEmbedAuthor, CreateEmbed, CreateSelectMenu, CreateActionRow, CreateSelectMenuOptions, CreateSelectMenuOption};

use mangadex_api::types::{Language, TagGroup};
use mangadex_api::types::{RelationshipType, ReferenceExpansionResource};
use mangadex_api::v5::schema::RelatedAttributes;
use mangadex_api::MangaDexClient;
use mangadex_api::CDN_URL;

use tracing::{info, error};

use regex::Regex;

use uuid::Uuid;

use url::Url;

use std::collections::HashMap;

use crate::utils::options::Options;
use ebina_macro::tracking;

const MANGADEX_COLOR: serenity::utils::Colour = Colour::from_rgb(246, 131, 40);

#[tracking("md_manga")]
#[command]
pub async fn manga(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let client = MangaDexClient::default();

	let mangadex_author = CreateEmbedAuthor::default().icon_url("https://i.imgur.com/gFzVv6g.png").name("MangaDex").url("https://mangadex.org/").clone();

    let title = args.rest();

    let manga_res = client
        .manga()
        .search()
        .title(title)
		.limit(10u32)
        .build()?
        .send()
        .await.unwrap();

	if manga_res.data.is_empty() {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.description("No results!");
				e.color(MANGADEX_COLOR);
				e.set_author(mangadex_author);
				e
			});
			m
		}).await?;
		return Ok(())
	}

	let mut options: Vec<String> = Vec::new();

	for m in &manga_res.data {
		options.push(m.attributes.title.values().next().unwrap().to_string())
	}

	let index = Options::new(ctx, msg)
		.title("Enter the number corresponding the Manga you want info about!")
		.options(options)
		.colour(MANGADEX_COLOR)
		.author(mangadex_author)
		.edit()
		.send()
		.await;
    let manga = manga_res.data[index.unwrap().0].clone();

	send_md_embed(ctx, msg, manga.id, true, Some(index.unwrap().1), Some(index.unwrap().2)).await;
    Ok(())
}

pub async fn manage_md_url(ctx: &Context, msg: &Message, url: Url) {
	let id_opt = {
		let mut path_segments = url.path_segments().ok_or("cannot be base").unwrap();
		if path_segments.next().unwrap() == "title" {
			path_segments.next()
		} else {
			return
		}
	};
	
	let id = match id_opt {
		Some(v) => Uuid::parse_str(v).unwrap(),
		None => return,
	};

	send_md_embed(ctx, msg, id, false, None, None).await;
}

async fn send_md_embed(ctx: &Context, msg: &Message, id: Uuid, edit: bool, message_id: Option<MessageId>, channel_id: Option<ChannelId>) {
	let client = MangaDexClient::default();

	let manga_res = client
		.manga()
		.get()
		.manga_id(&id)
		.includes(vec![ReferenceExpansionResource::Author, ReferenceExpansionResource::Artist])
		.build()
		.unwrap()
		.send()
		.await
		.unwrap();

	let manga = manga_res.data.clone();

	info!("{:#?}", manga);

	let mangadex_author = CreateEmbedAuthor::default().icon_url("https://i.imgur.com/gFzVv6g.png").name("MangaDex").url("https://mangadex.org/").clone();


	let manga_title = manga
        .attributes
        .title
		.values()
		.next()
        .unwrap();
    let manga_description = manga
        .attributes
        .description
        .get_key_value(&Language::English);
    let manga_cover_id = manga
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt)
        .expect("no cover art found for manga")
        .id;
    let manga_cover = client
        .cover()
        .get()
        .cover_id(&manga_cover_id)
        .build().unwrap()
        .send()
        .await.unwrap();
    let manga_authors = manga
        .relationships
        .iter()
        .filter(|related| related.type_ == RelationshipType::Author)
        .collect::<Vec<_>>();

    let manga_artists = manga
        .relationships
        .iter()
        .filter(|related| related.type_ == RelationshipType::Artist)
        .collect::<Vec<_>>();

	let manga_genres = manga.attributes.tags.iter().filter(|tag| tag.attributes.group == TagGroup::Genre).collect::<Vec<_>>();

	let manga_theme = manga.attributes.tags.iter().filter(|tag| tag.attributes.group == TagGroup::Theme).collect::<Vec<_>>();

	let manga_format = manga.attributes.tags.iter().filter(|tag| tag.attributes.group == TagGroup::Format).collect::<Vec<_>>();

	let mut embed = CreateEmbed::default();
	info!("{:#?}", manga_title);
	embed.title(manga_title);
	embed.color(Colour::from_rgb(246, 131, 40));
	embed.url(format!("https://mangadex.org/title/{}", manga.id));
	embed.thumbnail(&format!(
		"{}/covers/{}/{}",
		CDN_URL, manga.id, manga_cover.data.attributes.file_name
	));

	if let Some(desc) = manga_description {
		embed.description(fix_description(desc.1));
	}
	embed.set_author(mangadex_author);

	if !manga_authors.is_empty() {
		embed.field(
			"Authors",
			manga_authors
				.iter()
				.map(|auth| {
					let attri = auth.attributes.as_ref().unwrap();
					match attri {
						RelatedAttributes::Author(a) => a.name.as_str(),
						_ => unreachable!(),
					}
				})
				.collect::<Vec<_>>()
				.join(", "),
			true,
		);
	}

	if !manga_artists.is_empty() {
		embed.field(
			"Artists",
			manga_artists
				.iter()
				.map(|auth| {
					let attri = auth.attributes.as_ref().unwrap();
					match attri {
						RelatedAttributes::Author(a) => a.name.as_str(),
						_ => unreachable!(),
					}
				})
				.collect::<Vec<_>>()
				.join(", "),
			true,
		);
	}

	if manga.attributes.publication_demographic.is_some() {
		embed.field(
			"Demographic",
			manga.attributes.publication_demographic.unwrap(),
			true,
		);
	}

	if !manga_genres.is_empty() {
		embed.field(
			"Genres",
			manga_genres
				.iter()
				.map(|tag| {
					tag.attributes.name.values().next().unwrap().as_str()
				})
				.collect::<Vec<_>>()
				.join(", "),
			true,
		);
	}

	if !manga_theme.is_empty() {
		embed.field(
			"Theme",
			manga_theme
				.iter()
				.map(|tag| {
					tag.attributes.name.values().next().unwrap().as_str()
				})
				.collect::<Vec<_>>()
				.join(", "),
			true,
		);
	}

	if !manga_format.is_empty() {
		embed.field(
			"Format",
			manga_format
				.iter()
				.map(|tag| {
					tag.attributes.name.values().next().unwrap().as_str()
				})
				.collect::<Vec<_>>()
				.join(", "),
			true,
		);
	}
	embed.field("Publication Status", manga.attributes.status, true);

	/*if manga.artists().len() > 0 {
		e.field("Artist", manga.artists().join(", "), true);
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

	// Check if edit is true and if so edit the message instead
	if edit {
		// Get message from message ID
		let mut mess = match ctx.http.get_message(channel_id.unwrap().into(), message_id.unwrap().into() ).await {
			Ok(msg) => msg,
			Err(why) => {
				error!("Error getting message: {:?}", why);
				return;
			}
		};
		let _ = mess.edit(&ctx, |m| m.set_embed(embed)).await;
	} else {
		let _ = &msg.channel_id.send_message(&ctx.http, |m| m.set_embed(embed)).await;
	}

}

pub struct MDLinkOptions {
	pub track: Vec<u64>,
	pub roles: HashMap<u64, u64>
}

pub struct MDLink {
	pub guild_id: Option<u64>,
	pub channel_id: Option<u64>,
	pub group_id: Option<u64>,
	pub options: MDLinkOptions
}

// Sets a announcement channel for the bot to post updates regarding a mangadex group
// Including new chapters, new series.
#[tracking("md_link")]
#[command]
#[description = "Sets a channel to post updates regarding a mangadex group"]
#[aliases("link")]
#[only_in("guilds")]
#[required_permissions("ADMINISTRATOR")]
pub async fn link(ctx: &Context, msg: &Message) -> CommandResult {

	// Get the channel ID from prompt
	// First get all the channels in the guild
	let guild_id = msg.guild_id.unwrap();
	let channels = guild_id.channels(ctx).await.unwrap();

	let mut channel_options = Vec::<CreateSelectMenuOption>::new();

	channels.iter().for_each(|channel| {
		// Return if the channel is not a text channel
		if channel.1.kind != ChannelType::Text {
			return;
		}

		let mut option = CreateSelectMenuOption::default();
		option.label(&channel.1.name);
		option.value(channel.1.id);
		
		channel_options.push(option);
	});

	// Create CreateSelectorMenu to select the channel
	let mut menu = CreateSelectMenu::default();
	menu.custom_id("md_link_channel_select");
	menu.placeholder("Select a channel");
	menu.options(|o| {
			o.set_options(channel_options);
			o
		}
	);
	menu.max_values(1);

	// Instance of CreateActionRow
	let mut action_row = CreateActionRow::default();
	action_row.add_select_menu(menu);


	// Send a message with an embed with a select menu of all the channels
	let _ = msg.channel_id.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title("Select a channel");
			e.description("Select the channel you want it to send messages to");
			//e.field("Channels", channel_names.join(", "), true);
			e
		});
		m.components(|c| {
			c.add_action_row(action_row);
			c
		})
	}).await;
	Ok(())
}

fn fix_description<S: Into<String>>(description: S) -> String {
    let bold = Regex::new(r"\[(|/)b\]").unwrap();
	let mut desc = description.into();
    let spoilers = Regex::new(r"\[spoiler\].*\[/spoiler\]").unwrap();
    let language = Regex::new(r"(\[b\]\[u\]|\[u\]\[b\]).*(\[/u\]\[/b\]|\[/b\]\[/u\])(\n| |\r\n)\[spoiler\].*(\[/spoiler\]|)").unwrap();
    let horizontal = Regex::new(r"\[hr\](\n|)").unwrap();
    desc = language.replace_all(&desc, "").to_string();
    desc = bold.replace_all(&desc, "**").to_string();
    desc = spoilers.replace_all(&desc, "").to_string();
    desc = horizontal.replace_all(&desc, "").to_string();
    desc = desc.replace("&quot;", "\"");
    if desc.len() > 1000 {
        desc = format!("{}...", &desc[..1000]);
    }
    desc
}
