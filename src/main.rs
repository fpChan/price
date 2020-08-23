use std::collections::HashMap;
use reqwest;
use serde::Deserialize;
use reqwest::Error;


#[derive(Deserialize, Debug)]
struct Data {
//    symbol: String,
    quotes: HashMap<String, HashMap<String, f64>>
}

#[derive(Deserialize, Debug)]
struct Res {
    data: Data,
//    metadata:HashMap<String,>
}


#[tokio::main]
async fn main() {
    let request_url = String::from("https://widgets.coinmarketcap.com/v2/ticker/4948/");

    loop {
        let price = get_price(&request_url);
        println!("{:?}", futures::executor::block_on(price).unwrap());
    }
}

async fn get_price(url: &String) -> Result<f64, Error> {
    let response: Res = reqwest::get(url).await?.json().await?;
    Ok(response.data.quotes.get("USD").unwrap().get("price").unwrap().clone())
}



//#[tokio::main]
//async fn main() -> Result<(), Error> {
//    loop{
//        let request_url = String::from("https://widgets.coinmarketcap.com/v2/ticker/4948/");
//        let response:Res = reqwest::get(&request_url).await?.json().await?;
//        println!("{:?}",response.data.quotes.get("USD").unwrap().get("price").unwrap() );
//    }
//    Ok(())
//}