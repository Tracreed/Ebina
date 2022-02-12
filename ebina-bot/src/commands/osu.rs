use crate::OsuClientContainer;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use serenity::utils::*;

use osu_v2::user::UserMethods;
use read_color::*;

use std::convert::TryInto;

use chrono::Duration;

use humantime::format_duration;

use num_format::{Locale, ToFormattedString};

#[command]
#[example = "Peppy"]
#[description = "Used to get information about a user playing Osu!"]
#[usage = "<username>"]
pub async fn user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut client = data.get::<OsuClientContainer>().unwrap().lock().await;
    let username = match args.single::<String>() {
        Ok(v) => v,
        Err(_) => { &msg.author.name }.to_string(),
    };
    let mode = args
        .single::<String>()
        .unwrap_or_else(|_| "osu".to_string());
    let users = client.search_user(username.clone()).await?;
    if users.user.data.is_empty() {
        let message = MessageBuilder::new()
            .push("No user called ")
            .push_mono_safe(&username)
            .push(" found.")
            .build();
        msg.channel_id.say(&ctx, message).await?;
        return Ok(());
    }
    let user = &client.get_user(users.user.data[0].id, mode.clone()).await?;
    let usermode = match &user.rank_history {
        Some(v) => v.mode.clone(),
        None => mode.clone(),
    };
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(&user.username);
                e.url(format!("https://osu.ppy.sh/users/{}/{}", user.id, &mode));
                e.description(format!(
                    "{} :flag_{}:, mode: {}",
                    user.country.name,
                    user.country.code.to_lowercase(),
                    usermode
                ));
                e.author(|a| {
                    a.icon_url("https://ss.fuyu.moe/tracreed/JtFLLQGWfaf-OORSzwNt.png");
                    a.name("osu!");
                    a.url("https://osu.ppy.sh");
                    a
                });
                if user.profile_colour.is_some() {
                    let mut colstr = user.profile_colour.to_owned().unwrap();
                    colstr.remove(0);
                    let col = rgb(&mut colstr.chars()).unwrap();
                    let color = Colour::from_rgb(col[0], col[1], col[2]);
                    e.color(color);
                } else {
                    e.color(Colour::from_rgb(240, 110, 170));
                }
                if user.avatar_url.starts_with('/') {
                    e.thumbnail(format!("https://osu.ppy.sh{}", &user.avatar_url));
                } else {
                    e.thumbnail(&user.avatar_url);
                }
                if user.statistics.global_rank.is_some() {
                    e.field(
                        "Global Ranking",
                        format!(
                            "#{}",
                            user.statistics
                                .global_rank
                                .unwrap()
                                .to_formatted_string(&Locale::en)
                        ),
                        true,
                    );
                }
                if user.statistics.rank.country.is_some() {
                    e.field(
                        "Country Ranking",
                        format!(
                            "#{}",
                            user.statistics
                                .rank
                                .country
                                .unwrap()
                                .to_formatted_string(&Locale::en)
                        ),
                        true,
                    );
                }
                if user.statistics.hit_accuracy > 0.0 {
                    e.field(
                        "Accuracy",
                        format!("{:.2}%", user.statistics.hit_accuracy),
                        true,
                    );
                }
                e.field("Level", user.statistics.level.current, true);
                if user.statistics.play_time.is_some() {
                    let dur =
                        Duration::seconds(user.statistics.play_time.unwrap().try_into().unwrap());
                    e.field(
                        "Total Play Time",
                        format_duration(dur.to_std().unwrap())
                            .to_string()
                            .replace("days", "d")
                            .replace("day", "d"),
                        true,
                    );
                }
                if user.statistics.pp > 0.0 {
                    e.field("pp", format!("{:.0}", user.statistics.pp), true);
                }
                if user.statistics.ranked_score > 0 {
                    e.field(
                        "Ranked Score",
                        user.statistics
                            .ranked_score
                            .to_formatted_string(&Locale::en),
                        true,
                    );
                }
                if user.statistics.play_count > 0 {
                    e.field(
                        "Play Count",
                        user.statistics.play_count.to_formatted_string(&Locale::en),
                        true,
                    );
                }
                if user.statistics.total_score > 0 {
                    e.field(
                        "Total Score",
                        user.statistics.total_score.to_formatted_string(&Locale::en),
                        true,
                    );
                }
                if user.statistics.total_hits > 0 {
                    e.field(
                        "Total hits",
                        user.statistics.total_hits.to_formatted_string(&Locale::en),
                        true,
                    );
                }
                if user.statistics.maximum_combo > 0 {
                    e.field(
                        "Maximum Combo",
                        user.statistics
                            .maximum_combo
                            .to_formatted_string(&Locale::en),
                        true,
                    );
                }
                if user.statistics.replays_watched_by_others > 0 {
                    e.field(
                        "Replays Watched by Others",
                        user.statistics
                            .replays_watched_by_others
                            .to_formatted_string(&Locale::en),
                        true,
                    );
                }
                e.field(
                    "Join Date",
                    chrono::DateTime::parse_from_str(&user.join_date, "%Y-%m-%dT%H:%M:%S%:z")
                        .unwrap(),
                    false,
                );
                e
            });
            m
        })
        .await?;
    Ok(())
}
