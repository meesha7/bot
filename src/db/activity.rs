use crate::prelude::*;
use serenity::model::prelude::*;

#[derive(Debug)]
pub struct ActivityEntry {
    pub user_id: i64,
    pub last_message: i64,
}

/// Update the last_message field for a user
pub async fn update_user_activity(guild_id: GuildId, user_id: UserId) -> Result<()> {
    let mut conn = connect().await?;

    let (gid, uid) = (guild_id.to_string(), user_id.to_string());

    let result = sqlx::query!(
        "SELECT last_message FROM activity WHERE guild_id == ?1 AND user_id == ?2;",
        gid,
        uid,
    )
    .fetch_optional(&mut conn)
    .await?;

    let current_date = chrono::offset::Local::now();
    let timestamp = current_date.timestamp();

    match result {
        Some(row) => {
            log::debug!("Update entry for user {}", uid);
            log::debug!("Last message: {}", row.last_message.unwrap());

            sqlx::query!(
                "UPDATE activity SET last_message = ?1 WHERE guild_id == ?2 AND user_id == ?3;",
                timestamp,
                gid,
                uid,
            )
            .execute(&mut conn)
            .await?;
        }
        None => {
            log::debug!("Create entry for user {}", uid);

            sqlx::query!(
                "INSERT INTO activity (guild_id, user_id, last_message) VALUES (?1, ?2, ?3);",
                gid,
                uid,
                timestamp,
            )
            .execute(&mut conn)
            .await?;
        }
    }

    Ok(())
}
