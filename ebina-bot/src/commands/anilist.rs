use chrono::Utc;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::utils::Colour;
use serenity::builder::CreateEmbedAuthor;

use html2md::parse_html;

use ebina_anilist::{search, search_specific, get_schedule, queries::queries::MediaType};

use crate::anilist_embed;
use ebina_macro::tracking;
use crate::utils::options::Options;

const ANI_LIST_COLOR: serenity::utils::Colour = Colour::from_rgb(43, 45, 66);
// Anilist author constants
const ANI_LIST_AUTHOR_NAME: &str = "Anilist";
const ANI_LIST_AUTHOR_URL: &str = "https://anilist.co/";
const ANI_LIST_AUTHOR_ICON_URL: &str = "https://anilist.co/img/icons/apple-touch-icon.png";

/// Searches Anlist including both manga and anime.
#[tracking("al_search")]
#[command("search")]
#[help_available(false)]
pub async fn anilist_search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	anilist_media(ctx, msg, args, None).await?;
	Ok(())
}

#[tracking("al_manga")]
#[command("manga")]
pub async fn anilist_manga(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	anilist_media(ctx, msg, args, Some(MediaType::Manga)).await?;
	Ok(())
}

#[tracking("al_anime")]
#[command("anime")]
pub async fn anilist_anime(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
	anilist_media(ctx, msg, args, Some(MediaType::Anime)).await?;
	Ok(())
}

#[tracking("al_schedule")]
#[command("schedule")]
pub async fn anilist_schedule(ctx: &Context, msg: &Message) -> CommandResult {
	//Convert the current time to UTC
	let today = Utc::now();
	//Get the schedule for the current day
	let results = get_schedule(today).await.unwrap();

	let schedule = results.page.unwrap().airing_schedules.unwrap();

	// Anilist author
	let ani_list_author = CreateEmbedAuthor::default()
		.icon_url(ANI_LIST_AUTHOR_ICON_URL)
		.name(ANI_LIST_AUTHOR_NAME)
		.url(ANI_LIST_AUTHOR_URL)
		.to_owned();

	// temp schedule
	let temp_schedule = schedule.clone();

	// Find the next airing
	let next_airing = temp_schedule.iter().find(|airing| {
		let airing_at = chrono::DateTime::<Utc>::from_utc(
			chrono::NaiveDateTime::from_timestamp(airing.as_ref().unwrap().airing_at as i64, 0),
			Utc,
		);
		airing_at > today
	});

	// Create vector of strings representing the airing schedule
	let mut schedule_strs: Vec<String> = Vec::new();
	for schedule in schedule {
		let mut schedule_str = String::new();
		// Parse airing at as SystemTime
		let s = std::time::UNIX_EPOCH + std::time::Duration::from_secs(schedule.as_ref().unwrap().airing_at.try_into().unwrap());
		// Parse as DateTime with UTC timezone
		let dt = chrono::DateTime::<Utc>::from(s);
		let airing_at_str = dt.format("%H:%M").to_string();
		schedule_str.push_str(airing_at_str.as_str());

		schedule_str.push_str(" - ");
		schedule_str.push_str(schedule.as_ref().unwrap().media.as_ref().unwrap().title.as_ref().unwrap().user_preferred.as_ref().unwrap());
		// Add label to next airing
		if let Some(airing) = next_airing {
			if schedule.as_ref().unwrap().media.as_ref().unwrap().id == airing.as_ref().unwrap().media.as_ref().unwrap().id {
				schedule_str.push_str(" **(Next)**");
			}
		}
		schedule_strs.push(schedule_str);
	}

	// send embed containing the schedule for the day
	let _ = msg.channel_id.send_message(ctx, |m| {
		m.embed(|e| {
			e.title("Anilist Schedule");
			// Description including todays date
			e.description(format!("Today's schedule: {}", today.format("%Y-%m-%d")));
			e.color(ANI_LIST_COLOR);
			e.timestamp(today);
			e.set_author(ani_list_author);
			e.field("Schedule", schedule_strs.join("\n"), false);
			e
		})
	}).await?;

	Ok(())
}

pub async fn anilist_media(ctx: &Context, msg: &Message, args: Args, media_type: Option<MediaType>) -> CommandResult {
	let title = args.rest();
	let media_list;
	let ani_list_author = CreateEmbedAuthor::default()
		.icon_url(ANI_LIST_AUTHOR_ICON_URL)
		.name(ANI_LIST_AUTHOR_NAME)
		.url(ANI_LIST_AUTHOR_URL)
		.to_owned();

	if media_type.is_none() {
		let results = search(title).await.unwrap();
		media_list = results.page.unwrap().media.unwrap();
	} else {
		let results = search_specific(title, media_type).await.unwrap();
		media_list = results.page.unwrap().media.unwrap();
	}

	if media_list.is_empty() {
		msg.channel_id.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.description("No results!");
				e.color(ANI_LIST_COLOR);
				e.set_author(ani_list_author);
				e
			});
			m
		}).await?;
		return Ok(())
	}

	let mut options: Vec<String> = Vec::new();
	for m in &media_list {
		let manga = m.as_ref().unwrap();
		let mut option_string = vec![manga.title.as_ref().unwrap().user_preferred.as_ref().unwrap().to_string()];
		option_string.push(format!(" ({})", manga.format.unwrap()));

		if manga.is_adult.is_some() && manga.is_adult.unwrap() {
			option_string.push(" (NSFW)".to_string());
		}
		options.push(option_string.join(""));
	}

	let options_title = match media_type {
		Some(m_type) => {
			match m_type {
				MediaType::Anime => "Anime".to_string(),
				MediaType::Manga => "Manga".to_string(),
			}
		},
		None => "Media".to_string(),
	};

	//let index = send_options(ctx, msg, format!("Enter the number corresponding the {} you want info about!", options_title.to_lowercase()), options, Some(ANI_LIST_COLOR), Some(ani_list_author.clone())).await;
	let index = Options::new(ctx, msg)
		.title(format!("Enter the number corresponding the {} you want info about!", options_title.to_lowercase()))
		.options(options)
		.colour(ANI_LIST_COLOR)
		.author(ani_list_author.clone())
		.edit()
		.send()
		.await;
	if index.is_none() {
		return Ok(())
	}
	let media = media_list[index.unwrap().0].as_ref().unwrap();
	let mess = &mut ctx.http.get_message(index.unwrap().2.0, index.unwrap().1.0).await?;


	mess.edit(&ctx.http, |m| {
		m.embed(|e| {
			e.title(media.title.as_ref().unwrap().user_preferred.as_ref().unwrap());
			e.url(format!("https://anilist.co/{}/{}", media.type_.unwrap().to_string().to_lowercase(), media.id));

			if media.description.is_some() {
				e.description(parse_html(media.description.as_ref().unwrap()));
			}

			anilist_embed!(media.format, "Format", e);

			anilist_embed!(media.type_, "Type", e);

			anilist_embed!(media.chapters, "Chapters", e);

			anilist_embed!(media.volumes, "Volumes", e);

			anilist_embed!(media.episodes, "Episodes", e);

			anilist_embed!(media.status, "Status", e);

			anilist_embed!(media.mean_score.is_some(), format!("{}%", media.mean_score.unwrap()) , "Mean Score", e);

			anilist_embed!(media.is_adult.unwrap(), "Yes", "NSFW", e);

			if !media.genres.as_ref().unwrap().is_empty() {
				let genres = media.genres.as_ref().unwrap().iter().map(|g| g.as_ref().unwrap().to_string()).collect::<Vec<String>>();
				e.field("Genres", genres.join(", "), true);
			}

			if let Some(start_date) = media.start_date {
				if start_date.year.is_some() && start_date.month.is_some() && start_date.day.is_some(){
					e.field(
						"Start Date",
						chrono::naive::NaiveDate::from_ymd(
							start_date.year.unwrap(),
							start_date.month.unwrap().try_into().unwrap(),
							start_date.day.unwrap().try_into().unwrap(),
						),
							true
					);
				}
			}

			if let Some(end_date) = media.end_date {
				if end_date.year.is_some() && end_date.month.is_some() && end_date.day.is_some(){
					e.field(
						"End Date",
						chrono::naive::NaiveDate::from_ymd(
							end_date.year.unwrap(),
							end_date.month.unwrap().try_into().unwrap(),
							end_date.day.unwrap().try_into().unwrap(),
						),
						true
					);
				}
			}
			e.thumbnail(media.cover_image.as_ref().unwrap().large.as_ref().unwrap());
			e.set_author(ani_list_author);
			e.color(ANI_LIST_COLOR);
			e
		});
		m
	}).await?;

	Ok(())
}