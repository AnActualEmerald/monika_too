#[macro_use]
extern crate diesel;
extern crate dotenv;

mod db;
mod models;
mod schema;
mod utils;

use utils::Replyable;

use discord::model::Event;
use discord::model::Permissions;
use discord::ChannelRef;
use discord::Discord;
use discord::State;
use dotenv::dotenv;
use mlua::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set");
    let config = db::Database::new(&db_url);

    // login with the bot token, will have to be changed to an environment variable or something later
    let disc = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Couldn't find token"))
        .expect("Login failed");

    // connect to discord
    let (mut connection, rdy) = disc.connect().expect("Unable to connect");
    let mut state = State::new(rdy);

    //load commands & set up lua environment
    let commands = load_commands();
    let lua = Lua::new();

    println!("Bot ready...");
    loop {
        let evnt = match connection.recv_event() {
            Ok(event) => event,
            Err(discord::Error::Closed(code, body)) => {
                println!("[Error] Connection closed with status {:?}: {}", code, body);
                break;
            }
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
                continue;
            }
        };
        state.update(&evnt);
        match evnt {
            Event::MessageCreate(message) => match state.find_channel(message.channel_id) {
                Some(ChannelRef::Public(server, channel)) => {
                    if message.author.bot {
                        // skip messages from bot users
                        continue;
                    }
                    let prefix = if let Some(prf) = config.get_prefix(server.id.0) {
                        prf
                    } else {
                        config.add_server(server.id.0);
                        "!".to_owned()
                    };
                    //TODO: Set up proper logging
                    println!("{} says: {}", message.author.name, message.content);
                    if message.content.starts_with(&prefix) {
                        let perms = server.permissions_for(channel.id, message.author.id);
                        let command: Vec<&str> = message
                            .content
                            .split_once(&prefix)
                            .expect("Didn't find a command")
                            .1
                            .split(" ")
                            .collect();
                        println!("Found command {}", command[0]);
                        match command[0] {
                            "test" => {
                                let _ = disc.send_message(
                                    message.channel_id,
                                    "This is a reply to the test.",
                                    "",
                                    false,
                                );
                            }
                            "prefix" => {
                                perms.contains(Permissions::ADMINISTRATOR);
                                config.set_prefix(server.id.0, command[1]);
                                message.reply(&disc, &format!("prefix changed to {}", command[1]))
                            }
                            "quit" => {
                                let _ =
                                    disc.send_message(message.channel_id, "Exiting...", "", false);
                                println!("Quitting.");
                                break;
                            }
                            c => {
                                if commands.contains_key(c) {
                                    let src = &commands.get(c).unwrap().script;
                                    let res = lua.load(&src).eval::<String>();
                                    println!("{}", res.unwrap());
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

struct Command {
    name: String,
    script: String,
}

impl Command {
    fn from_file(path: &str) -> Command {
        if let Ok(src) = fs::read_to_string(path) {
            return Command {
                name: Path::new(path)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                script: src,
            };
        } else {
            panic!("Couldn't find command file at {}", path);
        }
    }
}

fn load_commands() -> HashMap<String, Command> {
    let mut coms: HashMap<String, Command> = HashMap::new();
    for entry in fs::read_dir("./commands").unwrap() {
        let path = entry.unwrap().path();
        coms.insert(
            path.file_stem().unwrap().to_str().unwrap().to_owned(),
            Command::from_file(path.to_str().unwrap()),
        );
    }

    coms
}
