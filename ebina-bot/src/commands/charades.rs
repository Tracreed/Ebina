use serenity::framework::standard::{macros::command, CommandResult};
use serenity::futures::stream::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::establish_connection;

use crate::models::{Categories, Difficulties};

use std::time::Duration;
use tracing::info;

use crate::diesel::prelude::*;
use crate::diesel::sql_types;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};

use crate::models::*;

#[command]
#[description = "A game similar to solving a rebus, figure out the anime/game from the emojis!"]
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

    let username = &ctx
        .http
        .get_user(results[0].userid.to_u64().unwrap())
        .await?
        .name;

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Guess that {:?}", results[0].category));
                e.description(&results[0].puzzle);
                e.field("Difficulty", format!("{:?}", results[0].difficulty), true);
                e.footer(|f| f.text(format!("Added by {}", username)))
            });

            m
        })
        .await?;

    let mut replies = msg
        .author
        .await_replies(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .build();

    let http = &ctx.http;

    loop {
        match replies.next().await {
            Some(reply) => {
                if reply.content.to_lowercase() == results[0].solution.to_lowercase() {
                    reply.reply(http, "You got it right!").await?;
                    replies.stop();
                    break;
                }
            }
            None => {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Time is up");
                            e.description(format!("The right answer was: {}", results[0].solution));
                            e
                        });

                        m
                    })
                    .await?;
                break;
            }
        }
    }
    Ok(())
}

#[command]
#[owners_only]
pub async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let mut question = MessageBuilder::new()
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
                msg.channel_id.say(&ctx.http, "Anime").await.unwrap();
                category = Categories::Anime;
            }
            "2" => {
                msg.channel_id.say(&ctx.http, "Game").await.unwrap();
                category = Categories::Game;
            }
            "3" => {
                msg.channel_id.say(&ctx.http, "Tv-Show").await.unwrap();
                category = Categories::TV;
            }
            "4" => {
                msg.channel_id.say(&ctx.http, "Movie").await.unwrap();
                category = Categories::Movie;
            }
            _ => {
                msg.channel_id.say(&ctx.http, "Try again").await.unwrap();
                return Ok(());
            }
        }
    };

    question = MessageBuilder::new()
        .push_line("What is the Difficulty? (Type the number of the corresponding category)")
        .push_line("1. Easy")
        .push_line("2. Medium")
        .push_line("3. Hard")
        .build();

    msg.channel_id.say(&ctx.http, question).await?;
    let mut difficulty = Difficulties::Easy;

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        match message.content.as_str() {
            "1" => {
                msg.channel_id.say(&ctx.http, "Easy").await.unwrap();
                difficulty = Difficulties::Easy;
            }
            "2" => {
                msg.channel_id.say(&ctx.http, "Medium").await.unwrap();
                difficulty = Difficulties::Medium;
            }
            "3" => {
                msg.channel_id.say(&ctx.http, "Hard").await.unwrap();
                difficulty = Difficulties::Hard;
            }
            _ => {
                msg.channel_id.say(&ctx.http, "Try again").await.unwrap();
                return Ok(());
            }
        }
    };

    info!("{:?}", category);
    let mut puzzle = String::from("");

    msg.channel_id.say(&ctx, "What is the puzzle?").await?;

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

    msg.channel_id.say(&ctx, "Hint? (y/n)").await?;

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        if message.content == *"y" {
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

    msg.channel_id.say(&ctx, "What is the solution?").await?;

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
        .push_line(format!("Solution is: {}", solution))
        .build();

    msg.channel_id.say(&ctx, response).await?;

    let conn = establish_connection();

    create_charade(
        &conn,
        NewCharade {
            category: &category,
            puzzle: puzzle.as_str(),
            hint: hint.as_str(),
            solution: solution.as_str(),
            difficulty: &difficulty,
            userid: &BigDecimal::from_u64(*msg.author.id.as_u64()).unwrap(),
            public: &true,
        },
    );

    Ok(())
}
use crate::schema::*;

pub fn create_charade(conn: &PgConnection, new_charade: NewCharade) -> Charade {
    diesel::insert_into(charades::table)
        .values(&new_charade)
        .get_result(conn)
        .expect("Error saving new post")
}
