use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[command]
#[required_permissions("BAN_MEMBERS")]
#[only_in("guilds")]
pub async fn ban(ctx: &Context, msg: &Message) -> CommandResult {
	let users = &msg.mentions;
	for user in users {
		let guild = msg.guild(&ctx).await.unwrap();
		let _ = match guild.ban(ctx, user, 1).await {
			Ok(v) => {v},
			Err(why) => {&msg.channel_id.say(&ctx, why).await?;},
		};
	}
	Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[only_in("guilds")]
pub async fn kick(ctx: &Context, msg: &Message) -> CommandResult {
	let users = &msg.mentions;
	for user in users {
		let _ = msg.guild(&ctx).await.unwrap().kick(&ctx, user.id).await.unwrap();
	}
	Ok(())
}

#[command]
#[aliases("uinfo")]
pub async fn userinfo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
	let guild = msg.guild(ctx).await.unwrap();
	let id = args.single::<u64>().unwrap_or(msg.author.id.0);
	let user = if &msg.mentions.len() < &1 {
		ctx.http.get_user(id).await?
	} else {
		msg.mentions[0].clone()
	};
	let gmember = guild.member(&ctx, &user.id).await.unwrap();
	let roles = gmember.roles(&ctx).await.unwrap_or(Vec::new());
	msg.channel_id
	.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title(&user.tag());
			let message = MessageBuilder::new()
			.mention(&user)
			.build();
			e.description(message);
			e.field("ID", format!("`{}`", &user.id), true);
			if roles.len() > 0 {
				let mut mess = MessageBuilder::new();
				for role in roles {
					mess.mention(&role);
					mess.push(" ");
				};

				e.field("Roles", mess.build(), true);
			}
			e.field("​Bot", format!("{}", gmember.user.bot), true);
			if gmember.joined_at.is_some() {
				e.field("Member since", format!("{}", &gmember.joined_at.unwrap().format("%a, %d %b %Y %T")), true);
			}
			e.field("Created at", gmember.user.id.created_at().format("%a, %d %b %Y %T"), true);
			e.thumbnail(&user.avatar_url().unwrap())
		});

		m
	})
	.await?;
	Ok(())
}

#[command]
#[aliases("ginfo")]
pub async fn guildinfo(ctx: &Context, msg: &Message) -> CommandResult {
	let guild = msg.guild(ctx).await.unwrap();
	msg.channel_id
	.send_message(&ctx.http, |m| {
		m.embed(|e| {
			e.title(&guild.name);
			e.field("ID", &guild.id, true);
			e.field("Member Count", &guild.member_count, true);
			e.field("Created", guild.id.created_at().format("%a, %d %b %Y %T"), true);
			e.field("Owner", guild.owner_id.mention(), true);
			//e.field("Roles", guild.roles, true);
			e.thumbnail(guild.icon_url().unwrap());
			e
		});

		m
	})
	.await?;
	Ok(())
}