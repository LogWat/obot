use serenity::{
    framework::standard::{
        macros::{command},
        CommandResult, Args,
    },
    model::{
        channel::GuildChannel,
        prelude::*,
    },
    prelude::*,
};

use crate::cache::{Database, SharedManagerContainer};
use crate::owner;
use crate::web::{
    api::{Api}, handler,
};

#[command]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let manager = data.get::<SharedManagerContainer>().cloned().unwrap();
    let mut manager = manager.lock().await;

    if owner::is_owner(&ctx, msg.author.id).await {
        msg.channel_id.say(&ctx.http, "Shutting down...").await?;
        manager.shutdown_all().await;
    } else {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
    }

    Ok(())
}

#[command]
async fn delmsg(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }

    let nd = match args.rest().parse::<u64>() {
        Ok(n) => n,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Invalid number").await?;
            return Ok(());
        }
    };

    let channel: GuildChannel = msg.channel_id.to_channel(&ctx.http).await?.guild().unwrap();
    let messages = match channel.messages(&ctx.http, |r| r.limit(nd)).await {
        Ok(m) => m,
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Failed to get messages").await?;
            return Ok(());
        }
    };
    if messages.len() == 0 {
        msg.channel_id.say(&ctx.http, "No messages").await?;
        return Ok(());
    }

    let mut ids = Vec::new();
    for m in messages {
        ids.push(m.id);
    }
    match channel.delete_messages(&ctx.http, &ids).await {
        Ok(_) => (),
        Err(_) => {
            msg.channel_id.say(&ctx.http, "Failed to delete messages").await?;
            return Ok(());
        }
    }

    msg.channel_id.say(&ctx.http, format!("Deleted {} messages", nd)).await?;
    Ok(())
}


// test command: todo
// if arg is "add", add todo, if arg is "list", print todo list, and 
// if arg is "remove", remove todo from database
#[command]
async fn todo(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    match args.current() {
        Some("add") => {
            let todo = msg.content.split("add ").collect::<Vec<&str>>()[1];
            let db = data.get::<Database>().cloned().unwrap();
            let db = db.lock().await;
            let user_id = msg.author.id.0 as i64;
            let _ = sqlx::query!(
                "INSERT INTO todo (user_id, todo) VALUES (?, ?)",
                user_id, todo)
                .execute(&*db)
                .await;
            msg.channel_id.say(&ctx.http, format!("Added todo: {}", todo)).await?;
        },
        Some("list") => {
            let db = data.get::<Database>().cloned().unwrap();
            let db = db.lock().await;
            let user_id = msg.author.id.0 as i64;
            let rows = sqlx::query!(
                "SELECT todo FROM todo WHERE user_id = ?",
                user_id)
                .fetch_all(&*db)
                .await?;
            let mut todos = String::new();
            for (i, row) in rows.iter().enumerate() {
                todos.push_str(&format!("{}. {}\n", i + 1, row.todo));
            }
            if todos.is_empty() {
                msg.channel_id.say(&ctx.http, "You have no todos").await?;
            } else {
                msg.channel_id.say(&ctx.http, format!(
                    "Todo list for {}:\n{}",
                    msg.author.name,
                    todos
                )).await?;
            }
        },
        Some("remove") => {
            let todo = msg.content.split("remove ").collect::<Vec<&str>>()[1];
            let db = data.get::<Database>().cloned().unwrap();
            let db = db.lock().await;
            let user_id = msg.author.id.0 as i64;
            let q = sqlx::query!(
                "SELECT todo FROM todo WHERE user_id = ? AND todo = ?",
                user_id, todo)
                .fetch_one(&*db)
                .await;
            match q {
                Ok(_) => {
                    let _ = sqlx::query!(
                        "DELETE FROM todo WHERE user_id = ? AND todo = ?",
                        user_id, todo)
                        .execute(&*db)
                        .await;
                    msg.channel_id.say(&ctx.http, format!("Removed todo: {}", todo)).await?;
                },
                Err(_) => {
                    msg.channel_id.say(&ctx.http, "Todo not found").await?;
                }
            }
        },
        _ => {
            msg.channel_id.say(&ctx.http, "Invalid argument").await?;
        }
    }

    Ok(())
}

// test command: get_maps
// fetch api and print maps
#[command]
async fn get_maps(ctx: &Context, msg: &Message) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    let api = Api::new();
    let token = match api.update_token().await {
        Ok(t) => t,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to update token: {}", e)).await?;
            return Ok(());
        }
    };
    let mode = "3"; // mania
    let status = "ranked";
    let maps = match api.get_beatmaps(&token, mode, status).await {
        Ok(m) => m,
        Err(e) => {
            msg.channel_id.say(&ctx.http, format!("Failed to get beatmaps: {}", e)).await?;
            return Ok(());
        }
    };
    let mut map_list = String::new();
    for map in maps {
        map_list.push_str(&format!("{} - {} [{}]\n", map.artist, map.title, map.star[0]));
    }
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Ranked mania maps");
            e.description(map_list);
            e
        });
        m
    }).await.expect("Failed to send message");
    Ok(())
}

#[command]
async fn update_database(ctx: &Context, msg: &Message) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    match handler::check_maps(ctx).await {
        Ok(_) => {},
        Err(_e) => {},
    }

    Ok(())
}