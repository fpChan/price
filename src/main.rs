mod config;
mod ckb_utils;
mod price;


use crate::config::load_price_config;
use crate::ckb_utils::{ send_tx};
use reqwest::Error;
use crate::price::{CoinMarketCap, Exchange};



#[tokio::main]
async fn main() -> Result<(),Error> {
    let ckb_config = load_price_config();
    let price_client = CoinMarketCap::default();

    let fee = ckb_config.fee_shannons;
    let mut input_hash = ckb_config.input_tx_hash;
    let loop_times = 20;
    let genesis_capacity: u64 = loop_times * fee ;
    let mut i: u64  = 1;

    while i < loop_times {
        // get price
        let  prices = price_client.get_ckb_price().await.unwrap();
        input_hash = send_tx(genesis_capacity -i*fee, input_hash.clone(), ckb_config.private_key.clone(), prices)
            .map_err(|err| format!("Send transaction error: {}", err)).unwrap();

        println!("hash : {}",input_hash);
        i += 1;
    }

    println!("the gas fee is not enough, there is {:?} shannons left", genesis_capacity -(i-1)*fee);
    Ok(())
}
