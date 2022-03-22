use crate::TagsContainer;
use crate::utils::options::Options;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::error::Error;
use tracing::{error, info};

use serde::{Deserialize, Serialize};
//use serde_json::Result;

use regex::Regex;

use vndb::protocol::message;

use isolang::Language;

#[derive(Serialize, Deserialize, Debug)]
pub struct VnTagJ {
    pub aliases: Vec<String>,
    pub applicable: bool,
    pub cat: String,
    pub description: String,
    pub id: u64,
    pub meta: bool,
    pub name: String,
    pub parents: Vec<i64>,
    pub searchable: bool,
    pub vns: i64,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VnTagD {
    ///ID.
    pub id: u64,
    ///Score from 0 to 3.
    ///
    ///Note that VNDB's scores are float numbers.
    pub score: f32,
    #[serde(rename = "spoiler level")]
    ///Spoiler severity.
    ///
    ///Possible values:
    ///* 0 - None;
    ///* 1 - Minor;
    ///* 2 - Major;
    pub spoiler: u8,
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
        message::Response::Ok => {}
        response => panic!("Unexpected response={:?}", response),
    }

    let vnns = match get_vn(title.trim().to_string(), client).await {
        Ok(v) => v,
        Err(why) => {
            error!("Error getting VN: {:?}", why);
            return Ok(());
        }
    };

    client = vnns.1;

    let vns = vnns.0;

    if vns.is_empty() {
        msg.reply(&ctx.http, "No results :(").await?;
        return Ok(());
    }

	let mut options: Vec<String> = Vec::new();

	for vn in &vns {
        options.push(vn.title.as_ref().unwrap().to_string());
    }

	let index = Options::new(ctx, msg)
		.title("Enter the number corresponding the Visual Novel you want info about!")
		.options(options)
		.edit()
		.send()
		.await;

	if index.is_none() {
		return Ok(())
	}

    let vn = &vns[index.unwrap().0];
	let mess = &mut ctx.http.get_message(index.unwrap().2.0, index.unwrap().1.0).await?;

    let releases = match get_release(vn.id, client).await {
        Ok(v) => v,
        Err(why) => {
            error!("Error Getting Releases: {:?}", why);
            return Ok(());
        }
    };

    let mut developers: Vec<vndb::protocol::message::response::results::ReleaseProducer> = vec![];
    for rel in releases {
        for producer in rel.producers {
            if producer.developer && !developers.iter().any(|dev| dev.id == producer.id) {
                developers.push(producer);
            }
        }
    }

    let mut devstring = Vec::<String>::new();

    for dev in developers {
        devstring.push(format!("[{}](https://vndb.org/p{})", dev.name, dev.id))
    }

    let data = ctx.data.write().await;
    let tags_data = data.get::<TagsContainer>().unwrap();

    let length = match vn.length {
        Some(v) => match v {
            1 => "Very Short (< 2 hours)",
            2 => "Short (2 - 10 hours)",
            3 => "Medium (10 - 30 hours)",
            4 => "Long (30 - 50 hours)",
            5 => "Very Long (> 50 hours)",
            _ => "",
        },
        None => "",
    };

    let mut languages = Vec::<String>::new();
    if !vn.languages.is_empty() {
        for lang in &vn.languages {
            if lang.len() > 3 {
                languages.push(lang_code_conv(lang.to_string()));
                break;
            }
			let is = Language::from_639_1(lang).unwrap();
            languages.push(is.to_name().to_string());
        }
    }

    let mut tags = Vec::<VnTagD>::new();

    for tag in &vn.tags {
        let new_tag = VnTagD {
            id: tag.id,
            score: tag.score,
            spoiler: tag.spoiler,
        };
        tags.push(new_tag);
    }

    tags.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let mut tags_string = Vec::<String>::new();
    tags.truncate(10_usize);

    if !tags.is_empty() {
        for tag in &tags {
            if tag.spoiler > 0 {
                continue;
            };
            let tag_data = match tags_data.get(&tag.id) {
                Some(v) => v,
                None => {
                    continue;
                }
            };
            tags_string.push(tag_data.name.clone());
        }
    }
    let re = Regex::new(r"\[url=(?P<url>.*?)\](?P<name>.*?)\[/url\]").unwrap();
    let phonere = Regex::new(r"(^\[|\]$)").unwrap();
    mess.edit(&ctx.http, |m| {
		m.embed(|e| {
			e.title(vn.title.as_ref().unwrap_or(&"".to_string()));
			if vn.description.is_some() {
				e.description(phonere.replace_all(
					&re.replace_all(vn.description.as_ref().unwrap(), "[$name]($url)"),
					"\\$1",
				));
			}
			e.thumbnail(vn.image.as_ref().unwrap_or(&"".to_string()));
			e.url(format!("https://vndb.org/v{}", vn.id));
			if vn.aliases.is_some() {
				e.field(
					"Aliases",
					vn.aliases.as_ref().unwrap().replace('\n', ", "),
					true,
				);
			}
			if !length.is_empty() {
				e.field("Length", length, true);
			}
			e.field("Popularity", vn.popularity, true);
			if vn.rating.is_some() {
				e.field(
					"Rating",
					format!("{} ({})", vn.rating.unwrap(), vn.votecount.unwrap()),
					true,
				);
			}
			if !devstring.is_empty() {
				e.field("Developers", devstring.join(" & "), true);
			}
			if !languages.is_empty() {
				e.field("Languages", languages.join(", "), true);
			}
			if vn.original.is_some() {
				e.field("Original title", vn.original.as_ref().unwrap(), true);
			}
			if !tags.is_empty() {
				e.field("Tags", tags_string.join(", "), true);
			}
			e
		});
		m
	})
	.await
	.unwrap();
    Ok(())
}

async fn get_vn(
    title: String,
    mut client: vndb::client::tokio::Client<tokio::net::TcpStream>,
) -> Result<
    (
        Vec<vndb::protocol::message::response::results::Vn>,
        vndb::client::tokio::Client<tokio::net::TcpStream>,
    ),
    Box<dyn Error>,
> {
    let get = message::request::Get {
        kind: message::request::get::Type::vn(),
        flags: message::request::get::Flags::new()
            .basic()
            .details()
            .tags()
            .stats(),
        filters: message::request::get::Filters::new().filter(format!(
            "search ~ \"{}\"",
            title.replace('\\', "").replace('\"', "\\\"")
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
            Ok((vnn, client))
        }

        vndb::protocol::Response::Ok => return Err("Recieved okay instead of vn".into()),
        vndb::protocol::Response::DBstats(_) => return Err("Recieved wrong type".into()),
        vndb::protocol::Response::Error(why) => {
            return Err(format!("Something went wrong: {}", why).into())
        }
    };
    vn
}

async fn get_release(
    id: u64,
    mut client: vndb::client::tokio::Client<tokio::net::TcpStream>,
) -> Result<Vec<vndb::protocol::message::response::results::Release>, Box<dyn Error>> {
    let get = message::request::Get {
        kind: message::request::get::Type::release(),
        flags: message::request::get::Flags::new().basic().producers(),
        filters: message::request::get::Filters::new().filter(format!("vn = \"{}\"", id)),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: Some(20),
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

        vndb::protocol::Response::Ok => return Err("Recieved okay instead of vn".into()),
        vndb::protocol::Response::DBstats(_) => return Err("Recieved wrong type".into()),
        vndb::protocol::Response::Error(why) => {
            return Err(format!("Something went wrong: {}", why).into())
        }
    };
    release
}

fn lang_code_conv(mut lang: String) -> String {
    lang = lang.to_lowercase();
    match lang.as_str() {
        "pt-br" => "Portuguese (Brazil)".to_string(),
        _ => "Unknown".to_string(),
    }
}
