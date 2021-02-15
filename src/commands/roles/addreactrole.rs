use crate::prelude::*;
use serenity::{
    collector::*,
    framework::standard::{macros::command, Args, CommandResult},
    futures::StreamExt,
    model::prelude::*,
    prelude::*,
};

#[command]
#[only_in(guilds)]
#[owners_only]
#[num_args(2)]
#[description("Makes the reaction to the message above add the role to a user.")]
#[usage("addreactrole <emoji> <role>")]
#[example("addreactrole :heart: Admin")]
async fn addreactrole(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let conn = connect()?;
    let x = msg
        .channel(ctx)
        .await
        .ok_or(anyhow!("Channel not found."))?
        .guild()
        .ok_or(anyhow!("Guild not found."))?
        .messages(ctx, |builder| builder.before(msg.id).limit(1))
        .await?;
    if x.is_empty() {
        return Ok(());
    };

    let gid = msg.guild_id.ok_or(anyhow!("Guild ID not found."))?;

    let reaction = match parse_reaction(&args.single::<String>()?, &gid, &ctx).await {
        Some(r) => r,
        None => return Ok(()),
    };

    let role = match parse_rol(&args.single::<String>()?, Some(&gid), Some(&ctx)).await {
        Some(rid) => match rid.to_role_cached(&ctx.cache).await {
            Some(r) => r,
            None => return Ok(()),
        },
        None => return Ok(()),
    };

    let reactionid = reaction.id;
    let parent_msg = x.get(0).ok_or(anyhow!("Parent message not found."))?.clone();
    parent_msg.react(&ctx, reaction).await?;

    let roleid = role.id;
    let mut collector = ReactionCollectorBuilder::new(&ctx)
        .message_id(parent_msg.id)
        .removed(true)
        .filter(move |reaction| match reaction.as_ref().emoji {
            ReactionType::Custom { id, .. } => id == reactionid,
            _ => false,
        })
        .await;

    msg.delete(&ctx).await?;

    sql_block!({
        let mut s = conn.prepare("INSERT INTO reactionroles (guild_id, message_id, role_id, reaction_id) values (?1, ?2, ?3, ?4)")?;
        s.execute(params![gid.to_string(), parent_msg.id.to_string(), roleid.to_string(), reactionid.to_string()])?;
    })?;

    let ctx = ctx.clone();
    tokio::spawn(async move {
        let http = &ctx.http;
        while let Some(event) = collector.next().await {
            match event.as_ref() {
                ReactionAction::Added(a) => {
                    let uid = match a.user_id {
                        Some(id) => id,
                        None => continue,
                    };
                    let gid = match a.guild_id {
                        Some(id) => id,
                        None => continue,
                    };
                    let guild = match gid.to_partial_guild(&http).await {
                        Ok(g) => g,
                        Err(_) => continue,
                    };
                    let mut member = match guild.member(&ctx, uid).await {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let _ = member.add_role(&http, roleid).await;
                }
                ReactionAction::Removed(r) => {
                    let uid = match r.user_id {
                        Some(id) => id,
                        None => continue,
                    };
                    let gid = match r.guild_id {
                        Some(id) => id,
                        None => continue,
                    };
                    let guild = match gid.to_partial_guild(&http).await {
                        Ok(g) => g,
                        Err(_) => continue,
                    };
                    let mut member = match guild.member(&ctx, uid).await {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let _ = member.remove_role(&http, roleid).await;
                }
            };
        }
    });

    Ok(())
}
