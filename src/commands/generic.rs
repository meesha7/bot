use crate::prelude::*;
use chrono::Datelike;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::env;

macro_rules! text_command {
    ($cname:ident, $action:expr, $desc:expr, $usage:expr, $example:expr, $extra:expr) => {
        #[command]
        #[only_in(guilds)]
        #[num_args(1)]
        #[description($desc)]
        #[usage($usage)]
        #[example($example)]
        async fn $cname(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
            let gid = msg.guild_id.ok_or_else(|| anyhow!("Guild ID not found."))?;

            let user_id = none_return_ok!(parse_user(&args.single::<String>()?, Some(&gid), Some(&ctx)).await);
            let author = gid.member(&ctx, msg.author.id).await?;
            let member = gid.member(&ctx, user_id).await?;

            msg.channel_id
                .say(
                    &ctx,
                    format!(
                        "***{}** {} **{}**{}*",
                        author.display_name(),
                        $action,
                        member.display_name(),
                        $extra
                    ),
                )
                .await?;

            Ok(())
        }
    };
}

#[command]
#[description = "Check if it's Wednesday"]
#[usage = "frog"]
#[example = "frog"]
async fn frog(ctx: &Context, msg: &Message) -> CommandResult {
    let frog_url = if let Ok(e) = env::var("FROG_URL") {
        e
    } else {
        String::from("Sorry, the frog cannot be found :(")
    };

    let current_date = chrono::offset::Local::now();
    let weekday = current_date.weekday();

    let message = if weekday == chrono::Weekday::Wed {
        frog_url
    } else {
        String::from("It's not Wednesday yet!")
    };

    msg.channel_id.say(&ctx, message).await?;

    Ok(())
}

text_command!(hug, "hugs", "Hugs another user.", "hug <user>", "hug Elinvynia", "");
text_command!(
    cuddle,
    "cuddles",
    "Cuddles another user.",
    "cuddle <user>",
    "cuddle Elinvynia",
    ""
);
text_command!(pat, "pats", "Headpats another user.", "pat <user>", "pat Elinvynia", "");
text_command!(
    headpat,
    "headpats",
    "Headpats another user.",
    "headpat <user>",
    "headpat Elinvynia",
    ""
);
text_command!(
    bonk,
    "bonks",
    "Bonks another user to horny jail.",
    "bonk <user>",
    "bonk Elinvynia",
    ". Go to horny jail!"
);
text_command!(
    boop,
    "boops",
    "boops another user.",
    "boop <user>",
    "boop Elinvynia",
    ""
);
