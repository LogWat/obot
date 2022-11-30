use std::{
    sync::{Arc},
    collections::{HashMap, HashSet},
};

use serenity::{
    client::bridge::gateway::ShardManager,
    model::{id::UserId},
    prelude::*,
};

use tokio::sync::Mutex;

// bot操作用の構造体(shutdownとか)
pub struct SharedManagerContainer;
impl TypeMapKey for SharedManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// コマンドの実行回数を記録する
pub struct CommandCounter;
impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

// オーナーのIDを記録する
pub struct Owners;
impl TypeMapKey for Owners {
    type Value = Arc<Mutex<HashSet<UserId>>>;
}

// データベースの接続情報を記録する
pub struct Database;
impl TypeMapKey for Database {
    type Value = Arc<Mutex<sqlx::SqlitePool>>;
}