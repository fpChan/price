use std::collections::{HashMap};
use reqwest;
use serde::Deserialize;
use reqwest::Error;
use std::ops::{Div, Mul};
use async_trait::async_trait;

#[async_trait]
pub trait Exchange {
    async fn get_ckb_price(&self)->Result<(u128,u128),Error>;
}

#[derive(Deserialize, Debug)]
struct Data {
    quotes: HashMap<String, HashMap<String, f64>>
}

#[derive(Deserialize, Debug)]
struct CmcResp {
    data: Data,
}

#[derive(Deserialize, Debug)]
pub struct CoinMarketCap {
    url: String
}

impl Default for CoinMarketCap{
    fn default() -> Self {
        CoinMarketCap{ url: "https://widgets.coinmarketcap.com/v2/ticker".to_owned() }
    }
}

#[async_trait]
impl Exchange for CoinMarketCap {
    async fn get_ckb_price(&self) ->  Result<(u128,u128),Error> {
        let eth_resp :CmcResp = reqwest::get(&format!("{}{}",self.url,"/1027/")).await?.json().await?;
        let btc_resp : CmcResp = reqwest::get(&format!("{}{}",self.url,"/1/")).await?.json().await?;
        let ckb_resp: CmcResp = reqwest::get(&format!("{}{}",self.url,"/4948/")).await?.json().await?;

        let eth_price = eth_resp.data.quotes.get("USD").unwrap().get("price").unwrap().clone();
        let btc_price = btc_resp.data.quotes.get("USD").unwrap().get("price").unwrap().clone();
        let ckb_price = ckb_resp.data.quotes.get("USD").unwrap().get("price").unwrap().clone();

        let base: f64 = 10 as f64;

        println!("ckb : {:?}, eth :{:?},  btc:{:?}", ckb_price,eth_price,btc_price);
        let ckb_wei  = ckb_price.mul(base.powf(18 as f64)).div(eth_price) as u128;
        let ckb_sat   = ckb_price.mul(base.powf(8 as f64)).div(btc_price) as u128;
        Ok((ckb_wei,ckb_sat))
    }
}

