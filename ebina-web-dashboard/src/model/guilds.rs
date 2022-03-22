use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guild {
    /// The unique Id identifying the guild.
    ///
    /// This is equivilant to the Id of the default role (`@everyone`) and also
    /// that of the default channel (typically `#general`).
    pub id: u64,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    pub owner_id: u64,
    /// The region that the voice servers that the guild uses are located in.
    #[deprecated(note = "Regions are now set per voice channel instead of globally.")]
    pub region: String,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the `InviteSplash` feature is enabled, this can be used to generate
    /// a URL to a splash image.
    pub splash: Option<String>,
    /// An identifying hash of the guild discovery's splash icon.
    ///
    /// **Note**: Only present for guilds with the `DISCOVERABLE` feature.
    pub discovery_splash: Option<String>,
    /// The ID of the channel to which system messages are sent.
    pub system_channel_id: Option<u64>,
    /// The id of the channel where rules and/or guidelines are displayed.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub rules_channel_id: Option<u64>,
    /// The id of the channel where admins and moderators of Community guilds
    /// receive notices from Discord.
    ///
    /// **Note**: Only available on `COMMUNITY` guild, see [`Self::features`].
    pub public_updates_channel_id: Option<u64>,
    /// The server's description, if it has one.
    pub description: Option<String>,
    /// The total number of users currently boosting this server.
    #[serde(default)]
    pub premium_subscription_count: u64,
    /// The guild's banner, if it has one.
    pub banner: Option<String>,
    /// The vanity url code for the guild, if it has one.
    pub vanity_url_code: Option<String>,
    /// The preferred locale of this guild only set if guild has the "DISCOVERABLE"
    /// feature, defaults to en-US.
    pub preferred_locale: String,
    /// Approximate number of members in this guild.
    pub approximate_member_count: Option<u64>,
    /// Approximate number of non-offline members in this guild.
    pub approximate_presence_count: Option<u64>,
    /// Whether or not this guild is designated as NSFW. See [`discord support article`].
    ///
    /// [`discord support article`]: https://support.discord.com/hc/en-us/articles/1500005389362-NSFW-Server-Designation
    #[deprecated(note = "Removed in favor of Guild::nsfw_level.")]
    #[serde(default)]
    pub nsfw: bool,
    /// The maximum amount of users in a video channel.
    pub max_video_channel_users: Option<u64>,
    /// The maximum number of presences for the guild. The default value is currently 25000.
    ///
    /// **Note**: It is in effect when it is `None`.
    pub max_presences: Option<u64>,
    /// The maximum number of members for the guild.
    pub max_members: Option<u64>,
    /// Whether or not the guild widget is enabled.
    pub widget_enabled: Option<bool>,
    /// The channel id that the widget will generate an invite to, or null if set to no invite
    pub widget_channel_id: Option<u64>,
    /// All active threads in this guild that current user has permission to view.
    #[serde(default)]
    pub threads: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guilds {
	pub guilds: Vec<Guild>
}