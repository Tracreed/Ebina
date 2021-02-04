//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
mod commands;

pub mod schema;
pub mod models;

use std::{
    collections::HashSet,
    env,
    sync::Arc,
};

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate bigdecimal;

use diesel::{
    prelude::*,
    pg::PgConnection
};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::macros::{
            group,
            help,
            command,
        },
        standard::{
            HelpOptions, help_commands,
            Args, CommandResult, CommandGroup,
            StandardFramework,
        }
    },
    http::Http,
    model::prelude::*,
    prelude::*,
};

use tracing::{error, info};
use tracing_subscriber::{
    FmtSubscriber,
    EnvFilter,
};

use self::models::{
    Charade,
    NewCharade
};

use commands::{
    general::*,
    owner::*,
    charades::*,
};

use bigdecimal::BigDecimal;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, &help_options, groups, owners).await;
    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        info!("Guilds in cache: {}", ctx.cache.guilds().await.len());
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(ping, quit)]
struct General;

#[group]
#[commands(play, add)]
struct Charades;

#[tokio::main]
async fn main() {
    // This will load the environment variables located at `./.env`, relative to
    // the CWD. See `./.env.example` for an example on how to structure this.
    dotenv::dotenv().expect("Failed to load .env file");

    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");


    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };
    
    // Create the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                   .owners(owners)
                   .prefix("~"))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&CHARADES_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_charade<'a>(conn: &PgConnection, category: &'a str, puzzle: &'a str, hint: &'a str, userid: &'a BigDecimal, public: &'a bool) -> Charade {
    use schema::charades;

    let new_charade = NewCharade {
        category,
        hint,
        puzzle,
        userid,
        public,
    };

    diesel::insert_into(charades::table)
        .values(&new_charade)
        .get_result(conn)
        .expect("Error saving new post")
}