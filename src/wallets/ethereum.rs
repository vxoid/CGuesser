use std::io;

use ethers::signers::LocalWallet;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Provider, Http, Middleware};

use hex::ToHex;
use rand::RngCore;
use tokio::runtime::Runtime;

use crate::wallet::Wallet;

const NODE_URL: &str = "https://mainnet.infura.io/v3/<your-infura-token>";

pub struct EthereumWallet {
  signer: SignerMiddleware<Provider<Http>, LocalWallet>,
  private_key: String,
  runtime: Runtime
}

impl Wallet for EthereumWallet {
  const SYMBOL: &'static str = "ETH";

  fn new(private_key: &str) -> std::io::Result<Self> where Self: Sized {
    let provider = Provider::<Http>::try_from(NODE_URL)
      .map_err(str_c_to_io_err)?;

    let wallet = private_key
      .parse::<LocalWallet>()
      .map_err(str_c_to_io_err)?;

    let signer = SignerMiddleware::new(provider, wallet);
    let runtime = Runtime::new()?;
    
    Ok(Self { signer, runtime, private_key: private_key.to_string() })
  }

  fn get_random() -> io::Result<Self> where Self: Sized {
    let mut rng = rand::rngs::OsRng;
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);

    let provider = Provider::<Http>::try_from(NODE_URL)
      .map_err(str_c_to_io_err)?;

    let wallet = LocalWallet::from_bytes(&key_bytes)
      .map_err(str_c_to_io_err)?;

    let signer = SignerMiddleware::new(provider, wallet);
    let runtime = Runtime::new()?;

    Ok(Self { signer, runtime, private_key: key_bytes.encode_hex() })
  }

  fn get_balances(&self) -> Vec<(String, io::Result<f64>)> {
    let future = self.signer.get_balance(self.signer.address(), None);
    let balance = self.runtime.block_on(future)
      .map(|balance| balance.as_u128() as f64 / 1e18)
      .map_err(str_c_to_io_err);

    vec![(self.signer.address().0.encode_hex(), balance)]
  }

  fn get_private(&self) -> String {
    self.private_key.clone()
  }
}

fn str_c_to_io_err<T>(error: T) -> io::Error where T: ToString {
  io::Error::new(
    io::ErrorKind::Other,
    error.to_string()
  )
}