use serenity::framework::standard::{macros::command, CommandResult};
use serenity::futures::stream::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::establish_connection;

use crate::schema::{Categories, Difficulties};

use std::time::Duration;
use tracing::info;

use crate::diesel::prelude::*;
use crate::diesel::sql_types;

use bigdecimal::{BigDecimal, FromPrimitive};

use crate::create_charade;
use crate::models::*;

#[command]
pub async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    use crate::schema::charades::dsl::*;

    let connection = establish_connection();

    no_arg_sql_function!(
        random,
        sql_types::Integer,
        "Represents the SQL RANDOM() function"
    );

    let results = charades
        .limit(1)
        .order(random)
        .load::<Charade>(&connection)
        .expect("Error loading posts");

    println!("Charade with ID {} selected", results[0].id);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Guess that {:?}", results[0].category));
                e.description(&results[0].puzzle)
            });

            m
        })
        .await?;

    let mut replies = msg
        .author
        .await_replies(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await;

    let http = &ctx.http;

    while let Some(reply) = replies.next().await {
        if reply.content == results[0].solution {
            reply
                .reply(http, "You got it right! Haha :D xD UwU")
                .await?;
            replies.stop();
            break;
        }
    }
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
    let mut category = Categories::Anime;

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
                category = Categories::Anime;
            }
            "2" => {
                &msg.channel_id.say(&ctx.http, "Game").await.unwrap();
                category = Categories::Game;
            }
            "3" => {
                &msg.channel_id.say(&ctx.http, "Tv-Show").await.unwrap();
                category = Categories::TV;
            }
            "4" => {
                &msg.channel_id.say(&ctx.http, "Movie").await.unwrap();
                category = Categories::Movie;
            }
            _ => {
                &msg.channel_id.say(&ctx.http, "Try again").await.unwrap();
            }
        }
    };

    info!("{:?}", category);
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

    let mut hint = String::from("");

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
        .push_line(format!("Category is: {:?}", category))
        .push_line(format!("Hint is: {}", hint))
        .push_line(format!("Puzzle is: {}", &puzzle))
        .push_line(format!("Solution is {}", solution))
        .build();

    &msg.channel_id.say(&ctx, response).await?;

    let conn = establish_connection();

    create_charade(
        &conn,
        &category,
        &puzzle.as_str(),
        &hint.as_str(),
        &solution.as_str(),
        &Difficulties::Easy,
        &BigDecimal::from_u64(*msg.author.id.as_u64()).unwrap(),
        &true,
    );

    Ok(())
}
