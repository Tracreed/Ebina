use serde::Serialize;
use serenity::model::guild::Guild;

#[derive(Serialize)]
pub struct Status {
	pub name: String,
	pub commands: u64,
}

#[derive(Serialize)]
pub struct Guilds {
	pub guilds: Vec<Guild>
}