use serenity::{
    model::{prelude::*},
    prelude::*,
};

use crate::cache::Owners;

pub async fn is_owner(ctx: &Context, user_id: UserId) -> bool {
    let data = ctx.data.read().await;
    let owners = data.get::<Owners>().cloned().unwrap();
    let owners = owners.lock().await;

    owners.contains(&user_id)
}