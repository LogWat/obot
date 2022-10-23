use std::{
    sync::{Arc},
};

use serenity::{
    client::bridge::gateway::ShardManager,
    prelude::*,
};

use tokio::sync::Mutex;

pub struct SharedManagerContainer;

impl TypeMapKey for SharedManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}