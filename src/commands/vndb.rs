use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::error::Error;
use tracing::{error, info};
use std::time::Duration;

use serde::{Deserialize, Serialize};
//use serde_json::Result;

use regex::Regex;

use vndb::protocol::message;

use isolang::Language;

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
#[min_args(1)]
#[example = "Steins;Gate"]
#[description = "Used to get information about a Visual Novel from vndb"]
#[usage = "<title>"]
pub async fn vn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.quoted();
    let mut title = String::from("");

    for arg in args.iter::<String>() {
        title.push_str(&format!(" {}", &arg.unwrap()));
    }
    info!("{}", title.trim());
    let vns = match get_vn(title.trim().to_string()).await {
        Ok(v) => v,
        Err(why) => {
            error!("Error getting VN: {:?}", why);
            return Ok(());
        }
    };

    info!("{}", vns.len());

    info!("{:?}", vns[0]);

    let mut vnsearch = Vec::<String>::new();

    for (i, vnn) in vns.iter().enumerate() {
        vnsearch.push(format!("{}. {}", i + 1, vnn.title.as_ref().unwrap()));
        if vns.len()-1 == i {
            vnsearch.push("Cancel".to_string());
        }
    }

    let embmsg = msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Enter the number corresponding the Visual Novel you want info about!");
                e.description(vnsearch.join("\n"))
            });

            m
        })
        .await?;
    //embmsg.delete(&ctx.http).await?;

    let mut vn: &message::response::results::Vn;

    let mut cancel = false;

    vn = &vns[0];

    if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
        .await
    {
        let num: i32;
        let lenvn = vns.len() as i32;
        num = match message.content.parse::<i32>() {
            Ok(v) => {v},
            Err(_) => {0},
        };
        if num <= lenvn && num > 0 {
            vn = &vns[((num-1) as usize)];
            message.delete(&ctx.http).await?;
        } else if message.content.to_lowercase() == "cancel".to_string() {
            cancel = true;
            message.delete(&ctx.http).await?;
        } 
    }
    if cancel {
        embmsg.delete(&ctx.http).await?;
        return Ok(())
    }
    
    embmsg.delete(&ctx.http).await?;

    let releases = match get_release(vn.id).await {
        Ok(v) => v,
        Err(why) => {
            error!("Error Getting Releases: {:?}", why);
            return Ok(());
        }
    };

    let mut developers: Vec<vndb::protocol::message::response::results::ReleaseProducer> = vec![];
    for rel in releases {
        for producer in rel.producers {
            if producer.developer {
                if !developers.iter().any(|dev| {
                    dev.id == producer.id
                }) {
                    developers.push(producer);
                }
            }
        }
    }

    info!("{:?}", developers);

    let mut devstring = Vec::<String>::new();

    for dev in developers {
        devstring.push(format!("[{}](https://vndb.org/p{})", dev.name, dev.id))
    }

    let length = match vn.length {
        Some(v) => {
            match v {
                1 => "Very Short (< 2 hours)",
                2 => "Short (2 - 10 hours)",
                3 => "Medium (10 - 30 hours)",
                4 => "Long (30 - 50 hours)",
                5 => "Very Long (> 50 hours)",
                _ => ""
            }
        }
        None => {""}
    };

    let mut languages = Vec::<String>::new();
    if vn.languages.len() > 0 {
        for lang in &vn.languages {
            let is;
            if lang.len() > 3 {
                languages.push(lang_code_conv(lang.to_string()));
                break;
            } else {
                is = Language::from_639_1(lang).unwrap();
            }
            languages.push(is.to_name().to_string());
        };
    }

    let re = Regex::new(r"\[url=(?P<url>.*)\](?P<name>.*)\[.*?\]").unwrap();

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(vn.title.as_ref().unwrap());
                if vn.description.is_some() {
                    e.description(re.replace_all(vn.description.as_ref().unwrap(), "[$name]($url)"));
                }
                e.thumbnail(vn.image.as_ref().unwrap());
                e.url(format!("https://vndb.org/v{}", vn.id));
                e.field("Developers", devstring.join(" & "), true);
                e.field("Length", length, true);
                if languages.len() > 0 {
                    e.field("Languages", languages.join(", "), true)
                } else {
                    e
                }
            });

            m
        })
        .await?;

    Ok(())
}

async fn get_vn(
    title: String,
) -> Result<Vec<vndb::protocol::message::response::results::Vn>, Box<dyn Error>> {
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

    let get = message::request::Get {
        kind: message::request::get::Type::vn(),
        flags: message::request::get::Flags::new().basic().details().tags(),
        filters: message::request::get::Filters::new().filter(format!(
            "search ~ \"{}\"",
            title.replace("\\", "").replace("\"", "\\\"")
        )),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: Some(5),
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

async fn get_release(id: u64) -> Result<Vec<vndb::protocol::message::response::results::Release>, Box<dyn Error>> {
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
    
    let get = message::request::Get {
        kind: message::request::get::Type::release(),
        flags: message::request::get::Flags::new().basic().producers(),
        filters: message::request::get::Filters::new().filter(format!("vn = \"{}\"", id)),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: None,
            sort: None,
            reverse: Some(true),
        }),
    };
    client.send(&get.into()).await.expect("Visual novel");

    let response = match client.receive().await {
        Ok(resp) => resp.unwrap(),
        Err(why) => panic!("Something went wrong when getting info: {}", why),
    };

    let release = match response {
        vndb::protocol::Response::Results(rel) => {
            let vnn = rel.release()?.items;
            Ok(vnn)
        }

        vndb::protocol::Response::Ok => Err("Recieved okay instead of vn")?,
        vndb::protocol::Response::DBstats(_) => Err("Recieved wrong type")?,
        vndb::protocol::Response::Error(why) => Err(format!("Something went wrong: {}", why))?,
    };
    release
}

fn lang_code_conv(mut lang: String) -> String {
    lang = lang.to_lowercase();
    match lang.as_str() {
        "pt-br" => {"Portuguese (Brazil)".to_string()}
        _ => {"Unknown".to_string()}
    }
}