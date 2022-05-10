use std::env;

use serenity::model::channel::Message;
use tracing::{error, info};

pub mod options;


/*/// # Description
/// Gets the group a command is in
pub async fn get_command_group(msg: &Message, command_name: &str) -> Option<String> {
	//let mut group = None;
	let mut iter = msg.content.split(' ');
	let command = iter.next()?;
	let mut prefix = env::var("PREFIX").expect("Expected a prefix in the environment");

	info!("{:?}", crate::GENERAL_GROUP);
	
	// Check if the command is in the general group command array
	if let Some(group) = crate::GENERAL_GROUP.commands.iter().find(|&g| g == command) {
		return Some(group.to_string());
	}



	info!("Command: {}", command);



	Some("Test".to_string())
}*/