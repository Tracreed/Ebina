mod models;
mod prometheus;

use tide::{Request, Response, Body};

use serenity::prelude::Context;
use serenity::model::id::GuildId;
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use crate::prometheus::prometheus_metrics;

#[derive(Clone)]
pub struct State {
	ctx: Context
}

impl State {
	pub fn new(ctx: Context) -> Self {
		State {
			ctx
		}
	}
}

pub struct WebApp {
	app: tide::Server<State>
}

impl WebApp {
	pub fn new(ctx: Context) -> Self {

		let cors = CorsMiddleware::new()
			.allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
			.allow_origin(Origin::from("*"))
			.allow_credentials(false);
		let mut app = tide::with_state(State::new(ctx));
		app.with(cors);
		app.at("/").get(hello);
		app.at("/api/status").get(status);
		app.at("/api/guilds").get(guilds);
		app.at("/api/guild/:id").get(get_guild);
		app.at("/metrics").get(prometheus_metrics);
		WebApp {
			app
		}
	}

	pub async fn start(self, address: String) -> tide::Result<()> {
		self.app.listen(address).await?;
		Ok(())
	}
}

async fn hello(req: Request<State>) -> tide::Result {
	let state = req.state();
	let ctx = &state.ctx;
	let user = ctx.cache.user(ctx.cache.current_user().await).await.unwrap();
	let guilds = ctx.cache.guilds().await;
    Ok(format!("Hello world!, {} is in {} guilds", user.name, guilds.len()).into())
}

async fn status(req: Request<State>) -> tide::Result {
	let state = req.state();
	let ctx = &state.ctx;
	let user = ctx.cache.user(ctx.cache.current_user().await).await.unwrap();

	//let st = user.;

	let status = models::Status {
		name: user.name,
		commands: 5,
	};

	let response = Response::builder(203)
		.body(Body::from_json(&status)?)
		.content_type("application/json")
		.build();

	Ok(response)
}

async fn guilds(req: Request<State>) -> tide::Result {
	let state= req.state();
	let ctx = &state.ctx;
	let guilds = ctx.cache.guilds().await;

	let mut gis = models::Guilds {
		guilds: Vec::new()
	};

	for gid in guilds {
		let guild = ctx.cache.guild(gid).await;
		println!("{}", gid);
		match guild {
			Some(g) => gis.guilds.push(g),
			None => continue,
		}
	}

	//println!("{:?}", guild);

	let response = Response::builder(203)
		.body(Body::from_json(&gis)?)
		.content_type("application/json")
		.build();
	Ok(response)
}

async fn get_guild(req: Request<State>) -> tide::Result {
	let state = req.state();
	let ctx = &state.ctx;
	let guild_id: u64 = req.param("id").unwrap().parse().unwrap();
	let gid = GuildId::from(guild_id);
	let guild = ctx.cache.guild(gid).await.unwrap();

	let json = serde_json::to_string(&guild).unwrap();

	let response = Response::builder(203)
		.body(json)
		.content_type("application/json")
		.build();

	Ok(response)
}