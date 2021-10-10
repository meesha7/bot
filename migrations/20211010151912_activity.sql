-- Add migration script here
CREATE TABLE activity (
    guild_id INTEGER,
    user_id INTEGER,
    last_message INTEGER,
    PRIMARY KEY (guild_id, user_id)
);
