use serenity::{
    model::{prelude::*},
    prelude::*,
};

use crate::cache::Owners;

pub async fn is_owner(ctx: &Context, user_id: UserId) -> bool {
    let data = ctx.data.read().await;
    let owners = match data.get::<Owners>().cloned() {
        Some(o) => o,
        None => {
            warn!("Owners not found in cache");
            return false;
        }
    };
    let owners = owners.lock().await;

    owners.contains(&user_id)
}