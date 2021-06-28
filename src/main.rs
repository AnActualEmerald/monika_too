#[macro_use]
extern crate diesel;
extern crate dotenv;

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
                        config.add_server(server.id.0 as i32);
                        "!".to_owned()
                    };
                    //TODO: Set up proper logging
                    println!("{} says: {}", message.author.name, message.content);
                    if message.content.starts_with(&prefix) {
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
                                config.set_prefix(server.id.0, command[1]);
                                message.reply(&disc, &format!("prefix changed to {}", command[1]))
                            }
                            "quit" => {
                                let _ =
                                    disc.send_message(message.channel_id, "Exiting...", "", false);
                                println!("Quitting.");
                                break;
                            }
                            _ => {
                                eprintln!("Didn't find command {}", &command[0]);
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
