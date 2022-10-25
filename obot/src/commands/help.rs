use serenity::{
    framework::standard::{
        help_commands,
        macros::{help},
        CommandResult, Args, HelpOptions, CommandGroup
    },
    model::prelude::*,
    prelude::*,
};

use std::collections::HashSet;

#[help]
async fn my_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(
        ctx, msg, args, help_options, groups, owners
    ).await;
    Ok(())
}