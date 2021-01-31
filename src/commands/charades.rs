use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    CommandResult,
    macros::command,
};
use serenity::{
    prelude::*,
    async_trait,
    model::prelude::*,
    utils::MessageBuilder,
    collector::MessageCollectorBuilder,
    futures::stream::StreamExt,
};

use std::time::Duration;
use tracing::{error, info};

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
    let mut gen = String::from("none");

    if let Some(genre) =&msg.author.await_reply(&ctx).channel_id(msg.channel_id).timeout(Duration::from_secs(60)).await {
        match genre.content.as_str() {
            "1" => {
                &msg.channel_id.say(&ctx.http, "Anime").await.unwrap();
                gen = String::from("Anime");
            },
            "2" => {
                &msg.channel_id.say(&ctx.http, "Game").await.unwrap();
                gen = String::from("Game");
            },
            "3" => {
                &msg.channel_id.say(&ctx.http, "Tv-Show").await.unwrap();
                gen = String::from("Tv-Show");
            },
            "4" => {
                &msg.channel_id.say(&ctx.http, "Movie").await.unwrap();
                gen = String::from("Movie");
            },
            _ => {
                &msg.channel_id.say(&ctx.http, "Try again").await.unwrap();
            },

        }
    };

    info!("{}", gen.as_str());

    let collector = MessageCollectorBuilder::new(&ctx)
        .author_id(msg.author.id)
        .channel_id(msg.channel_id)
        .collect_limit(1)
        .timeout(Duration::from_secs(60))
        .await;
    
    let http = &ctx.http;
    let response: Vec<_> = collector.then(|msg| async move {
        match msg.content.as_str() {
            "1" => msg.reply(http, "Anime").await.unwrap(),
            _ => msg.reply(http, "Try again").await.unwrap(),
        };
        let _ = msg.reply(http, "test").await;

        msg
    }).collect().await;

    //println!("{:?}", response);

    Ok(())
}