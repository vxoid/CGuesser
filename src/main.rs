mod worker;
mod wallet;
mod wallets;

use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use worker::factory::WorkerFactory;
use teloxide::{types, requests::{ResponseResult, Requester}, ApiError, Bot, RequestError};

const TOKEN: &str = "6521524969:AAHZdedGEAMS3-_UeGuME82q2HK2sa-58fk";

#[derive(Default)]
struct BotData {
  data: Arc<Mutex<HashMap<types::UserId, WorkerFactory>>>
}

impl BotData {
  pub fn new() -> Self {
    Self { data: Arc::new(Mutex::new(HashMap::new())) }
  }

  async fn handle_start(&mut self, bot: Bot, message: types::Message) -> ResponseResult<()> {
    let user_id = message.from().map(|user| user.id).ok_or(teloxide::RequestError::Api(ApiError::UserNotFound))?;

    if self.data.lock().unwrap().get(&user_id).is_none() {
      bot.send_message(message.chat.id, "Started few factory workers! You'll be notified about each error, etc.").await?;
      self.data.lock().unwrap().insert(user_id, self.create_factory(bot, message.chat.id));
    }

    Ok(())
  }

  async fn handle_stop(&mut self, bot: Bot, message: types::Message) -> ResponseResult<()> {
    let user_id = message.from().map(|user| user.id).ok_or(teloxide::RequestError::Api(ApiError::UserNotFound))?;

    let factory = self.data.lock().unwrap().remove(&user_id);
    if let Some(factory) = factory {
      bot.send_message(message.chat.id, "Stoping all factory workers... This might take some time.").await?;
      factory.exit();
    }

    Ok(())
  }

  fn create_factory(&self, bot: Bot, chat_id: types::ChatId) -> WorkerFactory {
    let (sender, mut receiver) = mpsc::unbounded_channel(); // Create an unbounded channel

    let closure = move |data: String, is_error: bool| {
      println!("{data}");

      if is_error {
        return;
      }

      if let Err(err) = sender.send(data) {
        println!("Error sending data through channel: {:?}", err);
      }
    };

    tokio::task::spawn(async move {
      while let Some(data) = receiver.recv().await {
        let bot = bot.clone();
        let _ = bot.send_message(chat_id, data.clone()).await;
      }
    });

    WorkerFactory::with_workers::<wallets::BitcoinWallet, _>(10, closure.clone())
      .add_workers::<wallets::EthereumWallet, _>(10, closure)
  }
}

#[tokio::main]
async fn main() {
  let bot = Bot::new(TOKEN);

  let bot_data = Arc::new(Mutex::new(BotData::new()));

  teloxide::repl(bot, move |bot: Bot, msg: types::Message| {
    let bot_data = bot_data.clone();
    
    tokio::task::spawn_blocking(move || {
      let mut bot_data = bot_data.lock().unwrap();
      
      match msg.text() {
        Some("/start") => {
          let future = bot_data.handle_start(bot, msg);
          tokio::runtime::Handle::current().block_on(future)?;
        },
        Some("/stop") => {
          let future = bot_data.handle_stop(bot, msg);
          tokio::runtime::Handle::current().block_on(future)?;
        }
        _ => {}
      };

      Ok::<(), RequestError>(())
    });

    async move {
      Ok(())
    }
  })
  .await;
}