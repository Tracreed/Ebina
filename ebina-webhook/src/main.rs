use actix_web::http::{header, Method, StatusCode};
use actix_web::{
    error, post, guard, middleware, web, App, Error, HttpRequest, HttpResponse,
    HttpServer, Result,
};

use futures::StreamExt;

use sha2::Sha256;

use hmac::{Hmac, Mac, NewMac};

extern crate dotenv;

type HmacSha256 = Hmac<Sha256>;

#[post("/rb")]
async fn reboot(mut payload: web::Payload) -> Result<HttpResponse> {
    let secret = std::env::var("GITEA_SECRET").expect("GITEA_SECRET needs to be set");
    let mut hmac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("Something went wrong with creating the HMAC");
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        hmac.update(&chunk.to_vec());
    }

    println!("Test");

Ok(HttpResponse::Ok().body(""))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().expect("Failed to load .env file");
    HttpServer::new(|| {
        App::new()
        .service(reboot)
    })
    .bind("0.0.0.0:25566")?
    .run()
    .await
}
