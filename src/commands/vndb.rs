use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::error::Error;
use tokio::net;
use tracing::{error, info};

use serde::{Deserialize, Serialize};
//use serde_json::Result;

use vndb::protocol::message;

#[derive(Serialize, Deserialize)]
struct Tag {
    aliases: Vec<String>,
    applicable: bool,
    cat: String,
    description: String,
    id: i64,
    meta: bool,
    name: String,
    parents: Vec<i64>,
    searchable: bool,
    vns: i64,
}

#[command]
pub async fn vn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.quoted();
    let mut title = String::from("");

    let mut client = vndb::client::tokio::Client::connect().await?;

    let login = vndb::protocol::message::request::Login {
        protocol: 1,
        client: "EbinaReed",
        clientver: 0.1,
        creds: None,
    };

    client.send(&login.into()).await.expect("send");

    match client
        .receive()
        .await
        .expect("To receive message")
        .expect("To not fail receiving")
    {
        message::Response::Ok => info!("Connected to VNDB api"),
        response => panic!("Unexpected response={:?}", response),
    }

    for arg in args.iter::<String>() {
        title.push_str(&format!(" {}", &arg.unwrap()));
    }
    info!("{}", title.trim());
    let vn = match get_vn(client, title.trim().to_string()).await {
        Ok(v) => v,
        Err(why) => {
            error!("{:?}", why);
            return Ok(());
        }
    };

    info!("{:?}", vn[0]);
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(vn[0].title.as_ref().unwrap());
                e.description(vn[0].description.as_ref().unwrap());
                e.thumbnail(vn[0].image.as_ref().unwrap());
                e.url(format!("https://vndb.org/v{}", vn[0].id));
                e.field("Developer", "[Key](https://vndb.org/p24) & Woop", true)
            });

            m
        })
        .await?;

    Ok(())
}

async fn get_vn(
    mut client: vndb::client::tokio::Client<net::TcpStream>,
    title: String,
) -> Result<Vec<vndb::protocol::message::response::results::Vn>, Box<dyn Error>> {
    let get = message::request::Get {
        kind: message::request::get::Type::vn(),
        flags: message::request::get::Flags::new().basic().details().tags(),
        filters: message::request::get::Filters::new().filter(format!(
            "search ~ \"{}\"",
            title.replace("\\", "").replace("\"", "\\\"")
        )),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: None,
            sort: Some("\"popularity\""),
            reverse: Some(true),
        }),
    };
    client.send(&get.into()).await.expect("Visual novel");

    let response = match client.receive().await {
        Ok(resp) => resp.unwrap(),
        Err(why) => panic!("Something went wrong when getting info: {}", why),
    };

    let vn = match response {
        vndb::protocol::Response::Results(vnb) => {
            let vnn = vnb.vn()?.items;
            Ok(vnn)
        }

        vndb::protocol::Response::Ok => Err("Recieved okay instead of vn")?,
        vndb::protocol::Response::DBstats(_) => Err("Recieved wrong type")?,
        vndb::protocol::Response::Error(why) => Err(format!("Something went wrong: {}", why))?,
    };
    vn
}

/*async fn get_release(id: i64) -> Result<Vec<vndb::protocol::message::response::results::Release>, Box<dyn Error>> {
    Ok(())
}*/
