use crate::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    let latency = match runner.latency {
        Some(latency) => latency,
        None => {
            &msg.reply(ctx, "Couldn't get latency. Maybe wait a little longer")
                .await?;

            return Ok(());
        }
    };

    msg.reply(&ctx.http, format!("Latency: {}ms", latency.as_millis()))
        .await?;
    Ok(())
}

#[command]
#[description = "Invite the bot to your server"]
pub async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let user = ctx.http.get_current_user().await?;
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Invite me");
            e.url(format!("https://discord.com/oauth2/authorize?client_id={}&permissions=0&scope=bot", user.id))
        });

        m
    }).await?;
    Ok(())
}
