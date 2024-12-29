use std::{collections::HashMap, sync::Arc};

use serenity::{
    all::{
        AuditLogEntry, ChannelId, Color, Context, CreateEmbed, CreateMessage, GuildId, Member,
        OnlineStatus, User, UserId,
    },
    gateway::ActivityData,
    model::{
        gateway::GatewayIntents,
        guild::audit_log::{Action, MemberAction},
    },
    Client,
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, serde::Deserialize)]
struct Config {
    pub transparency: u64,
    pub membership: u64,
}

#[derive(Clone, Debug, Default)]
struct Cache {
    banned_users: Vec<UserId>,
    kicked_users: Vec<UserId>,
}

struct Eikumi {
    pub configs: HashMap<GuildId, Config>,
    pub caches: Arc<Mutex<HashMap<GuildId, Arc<Mutex<Cache>>>>>,
}

unsafe impl Send for Eikumi {}
unsafe impl Sync for Eikumi {}

impl Eikumi {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            configs: serde_json::from_str(&std::fs::read_to_string("config.json")?)?,
            caches: Default::default(),
        })
    }

    pub async fn get_cache(&self, guild_id: GuildId) -> Arc<Mutex<Cache>> {
        (*self
            .caches
            .lock()
            .await
            .entry(guild_id)
            .or_insert_with(|| Default::default()))
        .clone()
    }

    pub fn get_transparency_channel(&self, guild_id: GuildId) -> Option<ChannelId> {
        self.configs
            .get(&guild_id)
            .map(|config| config.transparency.into())
    }

    pub fn get_membership_channel(&self, guild_id: GuildId) -> Option<ChannelId> {
        self.configs
            .get(&guild_id)
            .map(|config| config.membership.into())
    }

    // pub async fn add_author_to_embed(ctx: &Context, embed: CreateEmbed) -> CreateEmbed {
    //     let me = ctx.http.get_current_user().await;

    //     if me.is_err() {
    //         return embed;
    //     }

    //     let me = me.unwrap();

    //     embed.author({
    //         let embed_author = CreateEmbedAuthor::new(me.display_name());

    //         me.avatar_url().map_or(embed_author.clone(), |avatar_url| {
    //             embed_author.icon_url(avatar_url)
    //         })
    //     })
    // }

    pub async fn send_transparency_message(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        embed: CreateEmbed,
    ) {
        self.get_transparency_channel(guild_id)
            .unwrap()
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await
            .unwrap();
    }

    pub async fn send_membership_message(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        embed: CreateEmbed,
    ) {
        self.get_membership_channel(guild_id)
            .unwrap()
            .send_message(&ctx.http, CreateMessage::new().embed(embed))
            .await
            .unwrap();
    }
}

enum LeaveReason {
    Banned,
    Kicked,
    Voluntary,
}

#[serenity::async_trait]
impl serenity::client::EventHandler for Eikumi {
    // async fn message(&self, ctx: Context, msg: Message) {
    //     todo!()
    // }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        self.send_membership_message(&ctx, new_member.guild_id, {
            let embed = CreateEmbed::new()
                .title(format!("{} joined the server", new_member.display_name()))
                .field("User ID", new_member.user.id.get().to_string(), true)
                .field("Username", new_member.user.name.clone(), true)
                .color(Color::DARK_GREEN);

            new_member
                .user
                .avatar_url()
                .map_or(embed.clone(), |avatar_url| embed.thumbnail(avatar_url))
        })
        .await;
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _member_data_if_available: Option<Member>,
    ) {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut leave_reason = LeaveReason::Voluntary;

        if self
            .get_cache(guild_id)
            .await
            .lock()
            .await
            .kicked_users
            .contains(&user.id)
        {
            leave_reason = LeaveReason::Kicked;

            let cache_arc = self.get_cache(guild_id).await;
            let mut cache = cache_arc.lock().await;

            let position = cache
                .kicked_users
                .iter()
                .position(|user_id| user_id == &user.id)
                .unwrap();
            cache.kicked_users.swap_remove(position);
        }

        if self
            .get_cache(guild_id)
            .await
            .lock()
            .await
            .banned_users
            .contains(&user.id)
        {
            leave_reason = LeaveReason::Banned;

            let cache_arc = self.get_cache(guild_id).await;
            let mut cache = cache_arc.lock().await;

            let position = cache
                .banned_users
                .iter()
                .position(|user_id| user_id == &user.id)
                .unwrap();
            cache.banned_users.swap_remove(position);
        }

        self.send_membership_message(&ctx, guild_id, {
            let embed = CreateEmbed::new()
                .title(match leave_reason {
                    LeaveReason::Voluntary => format!("{} left the server", user.display_name()),
                    LeaveReason::Banned => format!("{} was banned", user.display_name()),
                    LeaveReason::Kicked => format!("{} was kicked", user.display_name()),
                })
                .field("User ID", user.id.get().to_string(), true)
                .field("Username", user.name.clone(), true)
                .color(match leave_reason {
                    LeaveReason::Voluntary => Color::RED,
                    LeaveReason::Banned => Color::DARK_RED,
                    LeaveReason::Kicked => Color::ORANGE,
                });

            user.avatar_url()
                .map_or(embed.clone(), |avatar_url| embed.thumbnail(avatar_url))
        })
        .await;
    }

    // async fn auto_moderation_action_execution(&self, ctx: Context, execution: ActionExecution) {
    //     todo!()
    // }

    async fn guild_audit_log_entry_create(
        &self,
        ctx: Context,
        entry: AuditLogEntry,
        guild_id: GuildId,
    ) {
        match entry.action {
            Action::Member(MemberAction::BanAdd) => {
                self.get_cache(guild_id)
                    .await
                    .lock()
                    .await
                    .banned_users
                    .push(entry.target_id.unwrap().get().into());

                let target: UserId = entry.target_id.unwrap().get().into();
                let target = target.to_user(&ctx).await.unwrap();

                self.send_transparency_message(&ctx, guild_id, {
                    let embed = CreateEmbed::new()
                        .title(format!("{} was banned", target.display_name()))
                        .color(Color::RED)
                        .field("User ID", target.id.get().to_string(), true)
                        .field("Username", target.name.clone(), true);

                    let embed = target
                        .avatar_url()
                        .map_or(embed.clone(), |avatar_url| embed.thumbnail(avatar_url));

                    entry
                        .reason
                        .map_or(embed.clone(), |reason| embed.field("Reason", reason, false))
                })
                .await;
            }
            Action::Member(MemberAction::BanRemove) => {
                let target: UserId = entry.target_id.unwrap().get().into();

                {
                    let cache_arc = self.get_cache(guild_id).await;
                    let mut cache = cache_arc.lock().await;

                    if let Some(idx) = cache.banned_users.iter().position(|user| user == &target) {
                        cache.banned_users.swap_remove(idx);
                    }
                }

                let target = target.to_user(&ctx).await.unwrap();

                self.send_transparency_message(&ctx, guild_id, {
                    let embed = CreateEmbed::new()
                        .title(format!("{} was unbanned", target.display_name()))
                        .color(Color::DARK_GREEN)
                        .field("User ID", target.id.get().to_string(), true)
                        .field("Username", target.name.clone(), true);

                    let embed = target
                        .avatar_url()
                        .map_or(embed.clone(), |avatar_url| embed.thumbnail(avatar_url));

                    entry
                        .reason
                        .map_or(embed.clone(), |reason| embed.field("Reason", reason, false))
                })
                .await;
            }
            Action::Member(MemberAction::Kick) => {
                self.get_cache(guild_id)
                    .await
                    .lock()
                    .await
                    .kicked_users
                    .push(entry.target_id.unwrap().get().into());

                let target: UserId = entry.target_id.unwrap().get().into();
                let target = target.to_user(&ctx).await.unwrap();

                self.send_transparency_message(&ctx, guild_id, {
                    let embed = CreateEmbed::new()
                        .title(format!("{} was kicked", target.display_name()))
                        .color(Color::ORANGE)
                        .field("User ID", target.id.get().to_string(), true)
                        .field("Username", target.name.clone(), true);

                    let embed = target
                        .avatar_url()
                        .map_or(embed.clone(), |avatar_url| embed.thumbnail(avatar_url));

                    entry
                        .reason
                        .map_or(embed.clone(), |reason| embed.field("Reason", reason, false))
                })
                .await;
            }
            Action::Member(MemberAction::Prune) => {
                // does not provide which users were pruned so... oof

                let pruned = entry
                    .options
                    .map_or(0, |options| options.members_removed.unwrap_or(0));

                if pruned == 0 {
                    return;
                }

                self.send_transparency_message(
                    &ctx,
                    guild_id,
                    CreateEmbed::new()
                        .title(format!(
                            "{} user{} {} pruned",
                            pruned,
                            if pruned == 1 { "" } else { "s" },
                            if pruned == 1 { "was" } else { "were" }
                        ))
                        .color(Color::PURPLE),
                )
                .await;
            }
            _ => (),
        }
    }

    // todo: dm user when they get kicked / banned
    // todo: timed bans, kicks, timeouts (NEED TO SEND EMBED FROM THE COMMAND — no audit log entry)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let token = std::env::var("DISCORD_TOKEN").expect("No DISCORD_TOKEN specified");

    let eikumi = Eikumi::new()?;

    println!("{:#?}", eikumi.configs);

    // panic!("We don't want to spam Discord now, do we?");

    let mut client = Client::builder(
        &token,
        GatewayIntents::default()
            | GatewayIntents::GUILD_PRESENCES
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .activity(ActivityData::watching("for rule violations!"))
    .status(OnlineStatus::DoNotDisturb)
    .event_handler(eikumi)
    .await?;

    if let Err(why) = client.start().await {
        println!("Err with client: {:?}", why);
    }

    Ok(())
}
