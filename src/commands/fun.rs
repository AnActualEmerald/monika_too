use rand::random;
use std::collections::HashMap;

use discord::{model::Message, Discord};

use crate::string;

pub fn get_commands<'a, 'b, 'c>() -> HashMap<String, Box<dyn Fn(&'a Discord, Message, Vec<String>)>>
{
    let mut commands: HashMap<String, Box<dyn Fn(&'a Discord, Message, Vec<String>)>> =
        HashMap::new();
    commands.insert(string!("joke"), Box::new(joke));
    commands
}

fn joke<'a, 'b, 'c>(client: &'a Discord, msg: Message, _args: Vec<String>) {
    //will read these from a file maybe
    let jokes = ["How do you fix a broken gorilla?\nWith a monkey wrench.", 
    "What’s the difference between a musician and a large pizza?\nA large pizza can feed a family of four.",
    "Why do police get to protests early?\nTo beat the crowd."
    ];

    let joke_num = random::<f32>() * jokes.len() as f32;

    client
        .send_message(msg.channel_id, jokes[joke_num as usize], "", false)
        .expect("Unable to send message");
}
