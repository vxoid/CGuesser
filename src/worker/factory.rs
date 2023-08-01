use std::collections::HashMap;

use crate::wallet;

use super::Worker;

#[derive(Default)]
pub struct WorkerFactory {
  symbols: HashMap<&'static str, usize>,
  workers: Vec<Worker>
}

impl WorkerFactory {
  pub fn with_workers<W, F>(amount: usize, debugger: F) -> Self
    where W: wallet::Wallet, F: FnMut(String, bool) + Clone + Send + 'static {
    let mut workers = Vec::with_capacity(amount);
    
    for i in 0..amount {
      workers.push(Worker::new::<W, F>(format!("{} {}", W::SYMBOL, i + 1), debugger.clone()))
    }

    let mut symbols = HashMap::new();
    symbols.insert(W::SYMBOL, amount);

    Self { workers, symbols }
  }

  pub fn add_workers<W, F>(mut self, amount: usize, debugger: F) -> Self
    where W: wallet::Wallet, F: FnMut(String, bool) + Clone + Send + 'static {
    let index_incr = *self.symbols.get(W::SYMBOL).unwrap_or(&0);

    for i in 0..amount {
      let index = index_incr + i + 1;

      let name = format!("{} {index}", W::SYMBOL);
      self.workers.push(Worker::new::<W, F>(name, debugger.clone()))
    }

    let new_amount = match self.symbols.get(W::SYMBOL) {
      Some(prev_amount) => prev_amount + amount,
      None => amount,
    };

    self.symbols.insert(W::SYMBOL, new_amount);

    self
  }

  pub fn exit(self) {
    for worker in self.workers {
      worker.exit()
    }
  }
}