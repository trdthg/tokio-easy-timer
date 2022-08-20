use dotenv::dotenv;
use std::sync::Arc;

use teloxide::{
    prelude::AutoSend,
    requests::{Request, Requester, RequesterExt},
    types::ChatId,
    Bot,
};
use tokio_easy_timer::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let bot = Arc::new(Bot::from_env().auto_send());
    let mut scheduler = scheduler::HeapScheduler::new();
    scheduler.add_ext(bot);

    scheduler.add(AsyncJob::new().every(10.seconds()).run(
        |bot: Data<Arc<AutoSend<Bot>>>| async move {
            bot.send_message(
                ChatId(std::env::var("CHAT_ID").unwrap().parse().unwrap()),
                format!("Hi!"),
            )
            .send()
            .await
            .unwrap();
        },
    ));
    scheduler.run_pending().await;
}
