use std::io;

use bitcoin::network::constants::Network;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{PrivateKey, PublicKey, Address};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::wallet::Wallet;

const NETWORK: Network = Network::Bitcoin;

type AddressCallback = fn(&PublicKey, Network) -> Result<Address, bitcoin::address::Error>;
const ADDRESS_TYPES: [AddressCallback; 3] = [
  Address::p2shwpkh,
  Address::p2wpkh,
  |pk, network| Ok(Address::p2pkh(pk, network))
];

#[derive(Clone)]
pub struct BitcoinWallet {
  private_key: PrivateKey,
  public_key: PublicKey
}

impl Wallet for BitcoinWallet {
  const SYMBOL: &'static str = "BTC";

  fn new(private_key: &str) -> std::io::Result<Self> where Self: Sized {
    let private_key = PrivateKey::from_wif(private_key)
      .map_err(btc_key_to_io_err)?;
  
    let public_key = private_key.public_key(&Secp256k1::new());

    Ok(Self { private_key, public_key })
  }

  fn get_random() -> io::Result<Self> where Self: Sized {
    let mut rng = OsRng;
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);

    let private_key = PrivateKey::from_slice(&key_bytes, NETWORK)
      .map_err(btc_key_to_io_err)?;
    
    let secp = Secp256k1::new();
    let public_key = private_key.public_key(&secp);
    
    Ok(Self { private_key, public_key })
  }

  fn get_balances(&self) -> Vec<(String, io::Result<f64>)> {
    let addresses = ADDRESS_TYPES
      .iter()
      .map(|address_fn| address_fn(&self.public_key, NETWORK))
      .filter(|result| result.is_ok())
      .map(Result::unwrap);

    addresses
      .map(|address| (address.to_string(), get_balance(&address.to_string()).map(|balance| balance as f64 / 1e8)))
      .collect()
  }

  fn get_private(&self) -> String {
    self.private_key.to_string()
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Transaction {
  txid: String,
  vout: i64,
  status: TransactionStatus,
  value: u64
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TransactionStatus {
  confirmed: bool,
  block_height: i64,
  block_hash: String,
}

fn get_balance(address: &str) -> io::Result<u64> {
  // GET https://blockstream.info/api/address/<wallet-address>/utxo
  // Example response:
  // [
  //   {
  //     "txid": "751af68087a748d680d4e82dcf26c3866ddbe8bf064497ca763927f8a1e14f3a",
  //     "vout": 0,
  //     "status": {
  //       "confirmed": true,
  //       "block_height": 800909,
  //       "block_hash": "00000000000000000003a4f5e79889b89f659ccaa25a9ecdc79496288ce4dda6",
  //       "block_time": 1690726853
  //     },
  //     "value": 187156
  //   }
  // ]
  let response = reqwest::blocking::get(format!("https://blockstream.info/api/address/{address}/utxo"))
    .map_err(reqwest_to_io_err)?
    .error_for_status()
    .map_err(reqwest_to_io_err)?;

  let utxos: Vec<Transaction> = response.json()
    .map_err(reqwest_to_io_err)?;

  let balance = utxos
    .iter()
    .filter(|tx| tx.status.confirmed)
    .map(|tx| tx.value)
    .sum();

  Ok(balance)
}

fn reqwest_to_io_err(error: reqwest::Error) -> io::Error {
  io::Error::new(
    io::ErrorKind::Other,
    error.to_string()
  )
}

fn btc_key_to_io_err(error: bitcoin::key::Error) -> io::Error {
  io::Error::new(
    io::ErrorKind::Other,
    error.to_string()
  )
}