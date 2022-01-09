use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;
use serenity::builder::CreateEmbedAuthor;

use mangadex_api::types::Language;
use mangadex_api::types::RelationshipType;
use mangadex_api::v5::schema::RelatedAttributes;
use mangadex_api::MangaDexClient;
use mangadex_api::CDN_URL;

use tracing::info;

use regex::Regex;

use html2md::parse_html;

const MANGADEX_COLOR: serenity::utils::Colour = Colour::from_rgb(246, 131, 40);

#[command]
pub async fn manga(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let client = MangaDexClient::default();

	let mangadex_author = CreateEmbedAuthor::default().icon_url("https://i.imgur.com/gFzVv6g.png").name("MangaDex").url("https://mangadex.org/").clone();

    let title = args.rest();

    let manga_res = client
        .manga()
        .search()
        .includes(vec![RelationshipType::Author, RelationshipType::Artist])
        .title(title)
		.limit(1u32)
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
    let manga = manga_res.data[0].clone();

    info!("{:#?}", manga);

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
        .build()?
        .send()
        .await?;
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

    let _ = &msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                info!("{:#?}", manga_title);
                e.title(manga_title);
                e.color(Colour::from_rgb(246, 131, 40));
                e.url(format!("https://mangadex.org/title/{}", manga.id));
                e.thumbnail(&format!(
                    "{}/covers/{}/{}",
                    CDN_URL, manga.id, manga_cover.data.attributes.file_name
                ));

				if let Some(desc) = manga_description {
					e.description(parse_html(desc.1));
				}
                e.set_author(mangadex_author);

                if !manga_authors.is_empty() {
                    e.field(
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
                    e.field(
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
                    e.field(
                        "Demographic",
                        manga.attributes.publication_demographic.unwrap(),
                        true,
                    );
                }

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
                e
            });
            m
        })
        .await?;
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
