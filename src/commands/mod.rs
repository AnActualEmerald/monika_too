mod fun;

use std::collections::HashMap;

use discord::{model::Message, Discord};

pub fn get_commands<'a>() -> HashMap<String, Box<dyn Fn(&'a Discord, Message, Vec<String>)>> {
    let mut commands: HashMap<String, Box<dyn Fn(&'a Discord, Message, Vec<String>)>> =
        HashMap::new();
    let fun_coms = fun::get_commands();

    for (k, v) in fun_coms.into_iter() {
        commands.insert(k, v);
    }

    commands
}
