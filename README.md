# obot (OSU! Discord BOT)
- This is a Discord bot that notifies beatmapset updates for osu!
- The development targets only the mania mode, but it can easily be extended to other game modes.

## How to use
- When you start this bot for the first time, initialize the database with the `init_database` command
- For details on available commands, please check the help command
- See .env_example for required environment variables
- Preparing...(I want to use Docker or something but the mapsets download function is in the way)

## Plans for future implementation
- Make downloaded maps available to all members of the same server (although there is the question of whether mapsets can be redistributed lol).
- Show pp information

## What this bot can do
- Automatically sends newly ranked, loved and (Qualified) beatmapsets as messages to the discord
- 4k and 7k mapsets can be sent per channel (or to the same channel, depending on the value of the environment variable)
- Automatically download beatmapsets above

## [!] Notice
If you find any problems with this bot, or if you have features you would like to see added, please send an issue to me. I welcome anyone who wants to help improve this bot with me! (I am new to bot development, Rust language and even osu!, so I'm sure there are a lot of mistakes lol)

## Folder Structure
```
obot/                           # the root of this cargo project
  ├── src/                      # code
  │    ├── main.rs              # main
  │    ├── cache.rs             # global data cache
  │    ├── owner.rs             # assistance with administrator-only functions
  |    ├── build.rs             # Scripts to run at build time
  │    ├── scheduler.rs         # Scheduler for mapsets update detection
  |    ├── utility.rs           # .env assistance
  |    ├── eventhandler.rs      # 
  |    ├── commands/            # commands
  |    |     ├── ...
  |    |
  |    ├── web/                 # osu!api handlers
  |    |     ├── ...
  |    |
  |    ├── db/                  # sqlite(sqlx) handlers
  |         ├── ...
  ├── migrations/               # migration of DB

```
