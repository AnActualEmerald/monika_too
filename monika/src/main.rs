#[macro_use]
extern crate diesel;
extern crate dotenv;

mod commands;
mod db;
mod models;
mod schema;
mod utils;

use utils::Replyable;

use discord::model::Event;
use discord::ChannelRef;
use discord::Discord;
use discord::State;
use dotenv::dotenv;
use std::env;

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
    let coms = commands::get_commands();
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
                Some(ChannelRef::Public(server, _channel)) => {
                    if message.author.bot {
                        // skip messages from bot users
                        continue;
                    }
                    let prefix = if let Some(cfg) = config.get_server_config(server.id.0) {
                        cfg[0].prefix.clone()
                    } else {
                        config.add_server(server.id.0);
                        string!("!")
                    };
                    //TODO: Set up proper logging
                    println!("{} says: {}", message.author.name, message.content);
                    let is_com = message.content.starts_with(&prefix);
                    if is_com {
                        let command: Vec<&str> = message
                            .content
                            .split_once(&prefix)
                            .expect("Didn't find a command")
                            .1
                            .split(" ")
                            .collect();
                        let args = if command.len() > 1 {
                            command[1].clone().split(" ").map(|e| string!(e)).collect()
                        } else {
                            vec![string!("")]
                        };
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
                                if let Some(command_func) = coms.get(c) {
                                    command_func(&disc, message, args);
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
