use serenity::{
    framework::standard::{
        macros::{command},
        CommandResult, Args,
    },
    model::prelude::*,
    prelude::*,
};

use crate::cache::{Database, SharedManagerContainer};
use crate::owner;

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