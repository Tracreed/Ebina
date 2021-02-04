use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::establish_connection;

use std::time::Duration;
use std::str::FromStr;
use tracing::{info};

use bigdecimal::BigDecimal;

use crate::create_charade;

#[command]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(&ctx, "test").await?;

    Ok(())
}

#[command]
#[owners_only]
pub async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let question = MessageBuilder::new()
        .push_line("What is the category? (Type the number of the corresponding category)")
        .push_line("1. Anime")
        .push_line("2. Game")
        .push_line("3. Tv-Show")
        .push_line("4. Movie")
        .build();

    msg.channel_id.say(&ctx.http, question).await?;
    let mut category = String::from("none");

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        match message.content.as_str() {
            "1" => {
                &msg.channel_id.say(&ctx.http, "Anime").await.unwrap();
                category = String::from("Anime");
            }
            "2" => {
                &msg.channel_id.say(&ctx.http, "Game").await.unwrap();
                category = String::from("Game");
            }
            "3" => {
                &msg.channel_id.say(&ctx.http, "Tv-Show").await.unwrap();
                category = String::from("Tv-Show");
            }
            "4" => {
                &msg.channel_id.say(&ctx.http, "Movie").await.unwrap();
                category = String::from("Movie");
            }
            _ => {
                &msg.channel_id.say(&ctx.http, "Try again").await.unwrap();
            }
        }
    };

    info!("{}", category.as_str());
    let mut puzzle = String::from("");

    &msg.channel_id.say(&ctx, "What is the puzzle?").await?;

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        puzzle = message.content.clone();
    };

    let mut hint= String::from("");

    &msg.channel_id.say(&ctx, "Hint? (y/n)").await?;

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        if message.content == String::from("y") {
            message.reply(&ctx.http, "What is the hint?").await?;
            if let Some(messg) = &msg
                .author
                .await_reply(&ctx)
                .channel_id(msg.channel_id)
                .timeout(Duration::from_secs(60))
                .await
            {
                hint = messg.content.clone();
            };
        }
    };

    let mut solution = String::from("");

    &msg.channel_id.say(&ctx, "What is the solution?").await?;  

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        solution = message.content.clone();
    }


    let response = MessageBuilder::new()
        .push_line(format!("Category is: {}", category))
        .push_line(format!("Hint is: {}", hint))
        .push_line(format!("Puzzle is: {}", &puzzle))
        .push_line(format!("Solution is {}", solution))
        .build();


    &msg.channel_id.say(&ctx, response).await?;

    let conn = establish_connection();

    create_charade(&conn, &category.as_str(), &puzzle.as_str(), &hint.as_str(), &solution.as_str(), &BigDecimal::from_str(&msg.author.id.to_string().as_str())?, &true);

    Ok(())
}
