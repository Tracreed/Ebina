use actix_web::web::Buf;
use actix_web::{
	post, web, App, HttpRequest, HttpResponse,
	HttpServer, Result,
};

use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use hex::encode;

use std::process::Command;

extern crate dotenv;

type HmacSha256 = Hmac<Sha256>;

#[post("/rb")]
async fn reboot(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse> {

	if !request.headers().get("Content-Type").unwrap().eq("application/json") {
		return Ok(HttpResponse::UnsupportedMediaType().body("415 Unsupported Media Type"))
	}

	let secret = std::env::var("GITEA_SECRET").expect("GITEA_SECRET needs to be set");

	let mut hmac = HmacSha256::new_from_slice(secret.as_bytes())
		.expect("Something went wrong with creating the HMAC");

	hmac.update(body.bytes());
	let hashed = encode(hmac.finalize().into_bytes());

	let gitea_hmac = request.headers().get("X-Gitea-Signature").unwrap();

	if gitea_hmac.eq(&hashed) {
		println!("git pull");
		Command::new("git")
			.arg("pull")
			.output()
			.expect("failed to git pull");
		println!("building");
		let cargo = Command::new("/home/ebina/.cargo/bin/cargo")
			.arg("build")
			.arg("--release")
			.output()
			.expect("failed to cargo build");
		println!("{:?}", cargo);
		println!("copying files");
		Command::new("sudo")
			.args(["systemctl", "stop", "ebina.service"])
			.output()
			.expect("failed to stop service");
		Command::new("cp")
			.arg("./target/release/ebina-bot")
			.arg("./bot")
			.output()
			.expect("failed to copy built bot");
		Command::new("cp")
			.arg("./target/release/ebina-webhook")
			.arg("./webhook")
			.output()
			.expect("failed to copy webhook binary");
		println!("restarting bot");
		Command::new("sudo")
			.arg("systemctl")
			.arg("start")
			.arg("ebina.service")
			.output()
			.expect("failed to restart service");
	} else {
		return Ok(HttpResponse::Unauthorized().body("415"))
	}

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
