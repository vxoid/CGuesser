pub mod factory;

use std::thread;
use std::sync;

use crate::wallet;

pub struct Worker {
  handler: thread::JoinHandle<()>,
  exit_signal_sender: sync::mpsc::Sender<()>,
}

impl Worker {
  pub fn new<W, F>(name: String, mut debugger: F) -> Self
    where W: wallet::Wallet, F: FnMut(String, bool) + Send + 'static {
    let (exit_signal_sender, exit_signal_receiver) = sync::mpsc::channel();
    
    let handler = thread::spawn(move || {
      loop {
        // Exit the thread if we caught exit signal
        if exit_signal_receiver.try_recv().is_ok() {
          debugger(format!("Closing {name} worker..."), false);
          return;
        }

        // Getting the wallet
        let wallet = match W::get_random() {
          Ok(wallet) => wallet,
          Err(err) => {
            debugger(format!("Worker '{name}' can't create wallet due to '{err}'"), true);
            continue;
          },
        };

        let balances = wallet.get_balances();

        for (public_key, balance) in balances {
          let balance = match balance {
            Ok(balance) => balance,
            Err(err) => {
              debugger(format!("Worker '{name}' can't get '{}' wallet balance due to '{err}'", public_key), true);
              continue;
            },
          };

          if balance > 0f64 {
            debugger(format!("'{}' has balance of {balance} {} at '{public_key}'", wallet.get_private(), W::SYMBOL), false);
          }
        }
      }
    });

    Self { handler, exit_signal_sender }
  }

  pub fn exit(self) {
    self.exit_signal_sender.send(()).expect("The exit channel is closed!");
    self.handler.join().expect("Can't join the thread");
  }
}