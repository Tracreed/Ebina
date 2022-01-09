use crate::OsuClientContainer;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use serenity::utils::*;
use std::env::temp_dir;
use std::fs::File;
use std::io::Write;

extern crate dotenv;

use tracing::info;

use osu_v2::user::UserMethods;
use read_color::*;

use magick_rust::{magick_wand_genesis, DrawingWand, MagickWand, PixelWand};

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

#[command]
pub async fn userimg(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let mut temp_file = temp_dir();
    temp_file.push(format!("{}", &user.id));
    temp_file.set_extension("jpeg");

    let mut tmp = std::fs::File::create(&temp_file).unwrap();

    let image_data = gen_image(user, usermode).await;

    tmp.write_all(&image_data)?;

    msg.channel_id
        .send_files(&ctx.http, vec![temp_file.to_str().unwrap()], |m| m)
        .await?;

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

async fn gen_image(user: &'_ osu_v2::user::User, usermode: String) -> Vec<u8> {
    magick_wand_genesis();
    info!("{}", &user.avatar_url);
    let avatar_url: String;
    if user.avatar_url.starts_with('/') {
        avatar_url = format!("https://osu.ppy.sh{}", &user.avatar_url);
    } else {
        avatar_url = user.avatar_url.clone();
    }
    let avatar_res = reqwest::get(avatar_url).await.unwrap();
    let mut dest = File::create(format!("./{}", &user.id)).unwrap();
    dest.write_all(&avatar_res.bytes().await.unwrap()).unwrap();
    let mut wand = MagickWand::new();

    wand.read_image("./assets/Ebina-osu-card.png").unwrap();

    let avatar = MagickWand::new();

    //avatar.set_size(512, 512).unwrap();

    avatar
        .read_image(format!("./{}", &user.id).as_str())
        .unwrap();
    avatar.resize_image(512, 512, 0);

    let avatar_round = MagickWand::new();
    avatar_round
        .read_image("./assets/avatar-remove.png")
        .unwrap();
    avatar.compose_images(&avatar_round, 2, true, 0, 0).unwrap();

    let mode_img = MagickWand::new();
    mode_img
        .read_image(format!("./assets/osu_icons/{}.png", usermode).as_str())
        .unwrap();
    mode_img.fit(51, 51);

    let progress_bar = MagickWand::new();
    progress_bar
        .read_image("./assets/progress-bar.png")
        .unwrap();

    let progress_bar_bg = MagickWand::new();
    progress_bar_bg
        .read_image("./assets/progress-bar-bg.png")
        .unwrap();
    let bar_width = ((progress_bar.get_image_width() / 100) as f64
        * user.statistics.level.progress as f64) as isize;
    info!("{}", bar_width);
    progress_bar_bg
        .compose_images(
            &progress_bar,
            2,
            true,
            (-(progress_bar.get_image_width() as isize) + bar_width) as isize,
            0,
        )
        .unwrap();

    let progress_bar_round = MagickWand::new();
    progress_bar_round
        .read_image("./assets/progress-bar-bg-remove.png")
        .unwrap();
    progress_bar_bg
        .compose_images(&progress_bar_round, 2, true, 0, 0)
        .unwrap();

    add_text(
        &mut wand,
        user.username.as_str(),
        347.0,
        611.0,
        "Torus",
        60.0,
        "white",
    );
    add_text(
        &mut wand,
        user.statistics.level.current.to_string().as_str(),
        291.0,
        788.0,
        "Torus",
        35.0,
        "white",
    );
    add_text(
        &mut wand,
        format!("{:.2}%", user.statistics.hit_accuracy).as_str(),
        1181.0,
        140.0,
        "Torus",
        35.0,
        "white",
    );
    let pp = user.statistics.pp as u64;
    add_text(
        &mut wand,
        pp.to_formatted_string(&Locale::en).as_str(),
        1594.0,
        140.0,
        "Torus",
        35.0,
        "white",
    );
    if user.statistics.global_rank.is_some() {
        add_text(
            &mut wand,
            format!(
                "#{}",
                user.statistics
                    .global_rank
                    .unwrap()
                    .to_formatted_string(&Locale::en)
            )
            .as_str(),
            1444.5,
            269.0 + 10.0,
            "Torus",
            35.0,
            "white",
        );
    } else {
        add_text(&mut wand, "-", 1444.5, 264.0 + 10.0, "Torus", 35.0, "white");
    }

    if user.statistics.rank.country.is_some() {
        add_text(
            &mut wand,
            format!(
                "#{}",
                user.statistics
                    .rank
                    .country
                    .unwrap()
                    .to_formatted_string(&Locale::en)
            )
            .as_str(),
            1444.5,
            411.0 + 10.0,
            "Torus",
            35.0,
            "white",
        );
    } else {
        add_text(&mut wand, "-", 1444.5, 411.0 + 10.0, "Torus", 35.0, "white");
    }

    if user.statistics.total_score > 0 {
        add_text(
            &mut wand,
            user.statistics
                .total_score
                .to_formatted_string(&Locale::en)
                .as_str(),
            1444.5,
            551.0 + 10.0,
            "Torus",
            35.0,
            "white",
        );
    } else {
        add_text(&mut wand, "-", 1444.5, 551.0 + 10.0, "Torus", 35.0, "white");
    }

    if user.statistics.maximum_combo > 0 {
        add_text(
            &mut wand,
            user.statistics
                .maximum_combo
                .to_formatted_string(&Locale::en)
                .as_str(),
            1699.5,
            691.0 + 10.0,
            "Torus",
            35.0,
            "white",
        );
    } else {
        add_text(&mut wand, "-", 1699.5, 691.0 + 10.0, "Torus", 35.0, "white");
    }

    if user.statistics.play_count > 0 {
        add_text(
            &mut wand,
            user.statistics
                .play_count
                .to_formatted_string(&Locale::en)
                .as_str(),
            1209.0,
            690.0 + 10.0,
            "Torus",
            35.0,
            "white",
        );
    } else {
        add_text(&mut wand, "-", 1209.0, 690.0 + 10.0, "Torus", 35.0, "white");
    }

    wand.compose_images(&avatar, 2, true, 91, 39).unwrap();
    wand.compose_images(&mode_img, 2, true, 1829, 28).unwrap();
    wand.compose_images(&progress_bar_bg, 2, true, 95, 800)
        .unwrap();
    wand.write_image_blob("JPEG").unwrap()
}

fn add_text(
    wand: &mut MagickWand,
    text: &str,
    x: f64,
    y: f64,
    font: &str,
    font_size: f64,
    color: &str,
) {
    let mut text_wand = DrawingWand::new();
    text_wand.set_font_family(font).unwrap();
    text_wand.set_font_size(font_size);

    let mut text_color = PixelWand::new();
    text_color.set_color(color).unwrap();

    text_wand.set_fill_color(&text_color);
    text_wand.set_text_alignment(2);
    wand.annotate_image(&text_wand, x, y, 0.0, text).unwrap();
}
