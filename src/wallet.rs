use std::io;

pub trait Wallet {
  const SYMBOL: &'static str;

  fn new(private_key: &str) -> io::Result<Self> where Self: Sized;
  fn get_balances(&self) -> Vec<(String, io::Result<f64>)>;
  fn get_random() -> io::Result<Self> where Self: Sized;
  fn get_private(&self) -> String;
}