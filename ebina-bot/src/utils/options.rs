use std::time::Duration;

use serenity::{utils::Colour, builder::CreateEmbedAuthor, model::{channel::Message, id::{MessageId, ChannelId}}, client::Context};

pub struct Options<'a> {
	ctx: &'a Context,
	msg: &'a Message,
	title: Option<String>,
	options: Vec<String>,
	colour: Option<Colour>,
	author: Option<CreateEmbedAuthor>,
	edit: bool,

}

impl<'a> Options<'a> {
	pub fn new(ctx: &'a Context, msg: &'a Message) -> Self {
		Options {
			ctx,
			msg,
			title: None,
			options: Vec::new(),
			colour: None,
			author: None,
			edit: false,
		}
	}

	/// Sets the title of the options
	pub fn title<S>(mut self, title: S) -> Self
	where S: Into<String> {
		self.title = Some(title.into());
		self
	}

	/// Sets single option
	pub fn option<S>(mut self, option: S) -> Self
	where S: Into<String> {
		self.options.push(option.into());
		self
	}

	/// Sets multiple options in one go
	pub fn options(mut self, options: Vec<String>) -> Self {
		for o in options {
			self.options.push(o);
		}
		self
	}

	/// Sets the colour of the embed
	pub fn colour(mut self, colour: Colour) -> Self {
		self.colour = Some(colour);
		self
	}

	/// Sets the author of the embed message
	pub fn author(mut self, author: CreateEmbedAuthor) -> Self {
		self.author = Some(author);
		self
	}

	/// If the embed should be for an edit or delete itself after the choice has been made
	pub fn edit(mut self) -> Self {
		self.edit = true;
		self
	}

	/// Sends the embed to the channel
	pub async fn send(self) -> Option<(usize, MessageId, ChannelId)> {
		let mut options_vec = Vec::<String>::new();

		for (i, option) in self.options.iter().enumerate() {
			options_vec.push(format!("{}. {}", i + 1, option))
		}
		options_vec.push("Cancel".to_string());

		let choice_embed = self.msg.channel_id
			.send_message(&self.ctx.http, |m| {
				m.embed(|e| {
					e.title(self.title.unwrap());
					e.description(options_vec.join("\n"));
					if let Some(c) = self.colour {
						e.color(c);
					}
					if let Some(a) = self.author {
						e.set_author(a);
					}
					e
				});
				m
			}).await.unwrap();

		let mut re_index = 0usize;

		let mut cancel = false;

		if options_vec.len() == 2 {
			return Some((re_index, choice_embed.id, choice_embed.channel_id));
		}

		if let Some(message) = &self.msg
			.author
			.await_reply(&self.ctx)
			.channel_id(self.msg.channel_id)
			.timeout(Duration::from_secs(60))
			.filter(|m| {
				if is_string_numeric(m.content.clone()) {
					true
				} else {
					m.content.to_lowercase().eq("cancel")
				}
			})
			.await
		{
			let lenvn = self.options.len() as i32;
			let num = message.content.parse::<i32>().unwrap_or(-1);
			if num <= lenvn && num > 0 {
				re_index = (num - 1) as usize;
				message.delete(&self.ctx.http).await.unwrap();
			} else if message.content.to_lowercase() == *"cancel" {
				cancel = true;
				message.delete(&self.ctx.http).await.unwrap();
			}
		}

		if cancel {
			choice_embed.delete(&self.ctx.http).await.unwrap();
			return None;
		}

		if !self.edit {
			choice_embed.delete(&self.ctx.http).await.unwrap();
		}
		Some((re_index, choice_embed.id, choice_embed.channel_id))
	}
}

pub async fn send_options(ctx: &Context, msg: &Message, title: String, options: Vec<String>, colour: Option<Colour>, author: Option<CreateEmbedAuthor>) -> Option<(usize, MessageId, ChannelId)> {
	let mut options_vec = Vec::<String>::new();

	for (i, option) in options.iter().enumerate() {
		options_vec.push(format!("{}. {}", i + 1, option))
	}
	options_vec.push("Cancel".to_string());

	let choice_embed = msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(title);
				e.description(options_vec.join("\n"));
				if let Some(c) = colour {
					e.color(c);
				}
				if let Some(a) = author {
					e.set_author(a);
				}
				e
			});
			m
		}).await.unwrap();

	let mut re_index = 0usize;

	let mut cancel = false;

	if options_vec.len() == 2 {
		return Some((re_index, choice_embed.id, choice_embed.channel_id));
	}

	if let Some(message) = &msg
        .author
        .await_reply(&ctx)
        .channel_id(msg.channel_id)
        .timeout(Duration::from_secs(60))
		.filter(|m| {
			if is_string_numeric(m.content.clone()) {
				true
			} else {
				m.content.to_lowercase().eq("cancel")
			}
		})
        .await
    {
        let lenvn = options.len() as i32;
        let num = message.content.parse::<i32>().unwrap_or(-1);
        if num <= lenvn && num > 0 {
            re_index = (num - 1) as usize;
            message.delete(&ctx.http).await.unwrap();
        } else if message.content.to_lowercase() == *"cancel" {
            cancel = true;
            message.delete(&ctx.http).await.unwrap();
        }
    }

	if cancel {
		choice_embed.delete(&ctx.http).await.unwrap();
		return None;
	}
	//choice_embed.delete(&ctx.http).await.unwrap();
    Some((re_index, choice_embed.id, choice_embed.channel_id))
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    true
}