mod models;

use tide::{Request, Response};

use serenity::prelude::Context;

#[derive(Clone)]
struct State {
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
		let mut app = tide::with_state(State::new(ctx));
		app.at("/").get(hello);
		app.at("/api/status").get(status);
		app.at("/api/guilds").get(guilds);
		app.at("/api/guild/:id").get(get_guild);
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

	let status = models::Status {
		name: user.name,
		commands: 5,
	};

	let json = serde_json::to_string(&status).unwrap();

	let response = Response::builder(203)
		.body(json)
		.content_type("application/json")
		.build();

	Ok(response)
}

async fn guilds(req: Request<State>) -> tide::Result {
	let state= req.state();
	let ctx = &state.ctx;
	let guilds = ctx.cache.guilds().await;
	let gis = models::Guilds{
		guilds: Vec::new()
	};

	let json = serde_json::to_string(&gis).unwrap();

	for gid in guilds {

	}

	//let guild = ctx.cache.guild(guilds[0]).await.unwrap();

	//println!("{:?}", guild);

	let response = Response::builder(203)
		.body(json)
		.content_type("application/json")
		.build();
	Ok(response)
}

async fn get_guild(req: Request<State>) -> tide::Result {
	let state = req.state();
	let ctx = &state.ctx;
	let guild_id = req.param("id").unwrap();
	let gid: u64 = guild_id.parse().unwrap();
	println!("{}", gid);
	let guild = ctx.cache.guild(gid).await.unwrap();

	let json = serde_json::to_string(&guild).unwrap();

	let response = Response::builder(203)
		.body(json)
		.content_type("application/json")
		.build();

	Ok(response)
}