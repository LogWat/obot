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

use crate::cache::{Database, SharedManagerContainer, CommandCounter};
use crate::owner;

#[command]
#[description("Bot-Processを終了します")]
async fn shutdown(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let manager = match data.get::<SharedManagerContainer>().cloned() {
        Some(manager) => manager,
        None => {
            error!("Failed to get manager(shutdown)");
            return Ok(());
        }
    };
    let mut manager = manager.lock().await;

    if owner::is_owner(&ctx, msg.author.id).await {
        msg.channel_id.say(&ctx.http, "Shutting down...").await?;
        info!("Shutting down by {}", msg.author.name);
        manager.shutdown_all().await;
    } else {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
    }

    Ok(())
}

#[command]
#[description("引数分だけコマンドが実行されたチャンネルのメッセージを削除します")]
#[min_args(1)]
#[max_args(1)]
async fn delmsg(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        info!("{} tried to use delmsg", msg.author.name);
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
#[description("todoを追加、削除、一覧表示します")]
#[usage("todo add <todo> | todo remove <todo> | todo list")]
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

// dbg command: print CommandCounter
#[command]
#[description("コマンドの実行回数を表示します")]
async fn infoc(ctx: &Context, msg: &Message) -> CommandResult {
    if owner::is_owner(&ctx, msg.author.id).await == false {
        msg.channel_id.say(&ctx.http, "You are not the owner").await?;
        return Ok(());
    }
    let data = ctx.data.read().await;
    let counter = data.get::<CommandCounter>().unwrap();
    let mut content = String::new();
    for (key, value) in counter.iter() {
        content.push_str(&format!("{}: {}\n", key, value));
    }

    match msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Command Counter");
            e.description(content);
            e
        });
        m
    }).await {
        Ok(_) => {},
        Err(e) => {
            warn!("Failed to send message in infoc: {}", e);
        }
    }

    Ok(())
}