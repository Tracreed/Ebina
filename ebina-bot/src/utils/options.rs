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
	/// # Examples
	/// ```rust,no_run
	/// use ebina_bot::utils::options::Options;
	/// 
	/// let mut options = Options::new(&ctx, &msg);
	/// options.title("Enter the number corresponding the Manga you want info about!");
	/// ```
	/// # Errors
	/// Returns an error if the title is too long
	/// # Panics
	/// Panics if the title is not set
	pub fn title<S>(mut self, title: S) -> Self
	where S: Into<String> {
		let title = title.into();
		if title.len() > 256 {
			panic!("Title is too long!");
		}
		self.title = Some(title);
		self
	}

	/// Sets single option
	/// # Examples
	/// ```rust
	/// use ebina_bot::utils::options::Options;
	/// 
	/// let options = Options::new(ctx, msg)
	///     .option("option1")
	///     .option("option2")
	///     .option("option3")
	///     .send()
	///     .await;
	/// ```
	/// # Errors
	/// Returns an error if the option is empty
	pub fn option<S>(mut self, option: S) -> Self
	where S: Into<String> {
		// Check if option is
		self.options.push(option.into());
		self
	}

	/// Sets multiple options in one go
	/// # Examples
	/// ```rust
	/// use ebina_bot::utils::options::Options;
	///
	/// let options = Options::new(ctx, msg)
	///    .options(vec!["option1", "option2", "option3"])
	///    .send()
	///    .await;
	/// ```
	/// # Errors
	/// Returns an error if any of the options are empty
	/// # Panics
	/// Panics if the options are empty
	pub fn options(mut self, options: Vec<String>) -> Self {
		for option in &options {
			if option.is_empty() {
				panic!("Option is empty!");
			}
		}
		self.options = options;
		self
	}

	/// Sets the colour of the embed
	/// # Examples
	/// ```rust
	/// use ebina_bot::utils::options::Options;
	/// use serenity::utils::Colour;
	///
	/// let options = Options::new(ctx, msg)
	///    .colour(Colour::from_rgb(255, 0, 0))
	///    .send()
	///    .await;
	/// ```
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
	///
	/// # Examples
	/// ```rust
	/// use ebina_bot::utils::options::Options;
	///
	/// let options = Options::new(ctx, msg)
	///    .edit()
	///   .send()
	///  .await;
	/// ```
	/// # Panics
	/// Panics if the edit is set to true and the message id is not set
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

		// Collect messages from user in the channel
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
			// Parse the reply as a number
			let num = message.content.parse::<i32>().unwrap_or(-1);
			if num <= lenvn && num > 0 {
				re_index = (num - 1) as usize;
				// Remove the reply from user
				message.delete(&self.ctx).await.ok();
			} else if message.content.to_lowercase().eq("cancel") {
				cancel = true;
				// Remove the reply from user
				message.delete(&self.ctx).await.ok();
			}
		}

		// Deletes the message if the user cancels
		if cancel {
			choice_embed.delete(&self.ctx).await.ok();
			return None;
		}

		// Deletes the message if it's not going to be edited
		if !self.edit {
			choice_embed.delete(&self.ctx).await.ok();
		}

		Some((re_index, choice_embed.id, choice_embed.channel_id))
	}
}

fn is_string_numeric(str: String) -> bool {
    for c in str.chars() {
        if !c.is_numeric() {
            return false;
        }
    }
    true
}