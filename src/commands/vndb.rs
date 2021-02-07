use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::error::Error;

use vndb::protocol::message;

#[command]
pub async fn vn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let title = args.single_quoted::<String>().unwrap();

    let get = message::request::Get {
        kind: message::request::get::Type::vn(),
        flags: message::request::get::Flags::new().basic().details(),
        filters: message::request::get::Filters::new().filter(format!("title ~ \"{}\"", title.replace("\\", "").replace("\"", "\\\""))),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: None,
            sort: None,
            reverse: Some(true)
        })
    };

    println!("{}", get);

    let mut login = vndb::protocol::message::request::Login{
        protocol: 1,
        client: "EbinaReed",
        clientver: 0.1,
        creds: None,
    };

    client.send(&login.into()).await.expect("send");
    client.send(&message::Request::DBstats).await.expect("stats");
    client.send(&get.into()).await.expect("Visual novel");

    /*match client.receive().await {
        message::Response::Ok => println!("Got back data!"),
        response => println!("Not good response"),
    }*/
    match client.receive().await.expect("To receive message").expect("To not fail receiving") {
        message::Response::Ok => println!("Ok"),
        response => panic!("Unexpected response={:?}", response),
    }

    /*match client.receive().await.expect("To receive message").expect("To not fail receiving") {
        message::Response::DBstats(response) => {
            println!("DBstats={:?}", response);
        },
        response => panic!("Unexpected response={:?}", response),
    }*/

    let info = match client.receive().await {
        Ok(resp) => {
            match resp {
                Some(response) => response,
                None => {
                    return Ok(())
                }
            }
        },
        Err(why) => panic!("Something went wrong when getting info: {}", why)
    };

    let vn = match client.receive().await {
        Ok(resp) => {
            match resp {
                Some(response) => response,
                None => {
                    return Ok(())
                }
            }
        },
        Err(why) => panic!("Something went wrong when getting info: {}", why)
    };

    println!("{:?}", info);
    match vn {
        vndb::protocol::Response::Ok => {}
        vndb::protocol::Response::Results(vnb)  => {
            let vnn = vnb.vn()?.items;
            println!("{:?}", vnn[0].title.as_ref().unwrap());
        },
        vndb::protocol::Response::DBstats(_) => {}
        vndb::protocol::Response::Error(_) => {}
    };
    Ok(())

}

async fn getvn(title: String) -> Result<Vec<vndb::protocol::message::response::results::Vn>, Box<Error>> {

    let mut client = vndb::client::tokio::Client::connect().await?;
    let get = message::request::Get {
        kind: message::request::get::Type::vn(),
        flags: message::request::get::Flags::new().basic().details(),
        filters: message::request::get::Filters::new().filter(format!("title ~ \"{}\"", title.replace("\\", "").replace("\"", "\\\""))),
        options: Some(message::request::get::Options {
            page: Some(1),
            results: None,
            sort: None,
            reverse: Some(true)
        })
    };

    let mut login = vndb::protocol::message::request::Login{
        protocol: 1,
        client: "EbinaReed",
        clientver: 0.1,
        creds: None,
    };

    client.send(&login.into()).await.expect("send");
    client.send(&get.into()).await.expect("Visual novel");

    let vn = match client.receive().await {
        Ok(resp) => {
            resp.unwrap()
        },
        Err(why) => panic!("Something went wrong when getting info: {}", why)
    };

    let som = match vn {
        vndb::protocol::Response::Results(vnb)  => {
            let vnn = vnb.vn()?.items;
            println!("{:?}", vnn[0].title.as_ref().unwrap());
            Ok(vnn)
        }
        vndb::protocol::Response::Ok => {}
        vndb::protocol::Response::DBstats(_) => {}
        vndb::protocol::Response::Error(_) => {}
    };
    som
}