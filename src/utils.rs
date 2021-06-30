use discord::model::Message;
use discord::Discord;

pub trait Replyable {
    fn reply(&self, client: &Discord, reply: &str);
}

impl Replyable for Message {
    fn reply(&self, context: &Discord, reply: &str) {
        let _ = context.send_message(
            self.channel_id,
            &format!("<@{}>, {}", self.author.id, reply),
            "",
            false,
        );
    }
}

#[macro_export]
macro_rules! string {
    ($a:literal) => {
        String::from($a)
    };
    ($a:expr) => {
        String::from($a)
    };
}

// #[macro_export]
// macro_rules! box {
//     ($a:expr) => {
//         Box::new($a)
//     };
// }
