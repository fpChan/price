use std::collections::{HashMap};
use reqwest;
use serde::Deserialize;
use reqwest::Error;
use std::ops::{Div, Mul, Add};

use ckb_sdk::{
    constants::{SIGHASH_TYPE_HASH},
    HttpRpcClient,GenesisInfo,
};
use ckb_types::core::BlockView;

use ckb_types::{
    bytes::Bytes,
    core::{ScriptHashType, TransactionBuilder, TransactionView},
    h160, h256,
    packed,
    prelude::*,
    H160, H256,
};

use ckb_crypto::secp::{Privkey};
use ckb_hash::{new_blake2b};
use faster_hex::{hex_decode};

const SIGNATURE_SIZE: usize = 65;
const CELL_CAPACITY: u64 = 240 * 100000000 ;

/*
account config
  address:
    mainnet: ckb1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqqyf57g
    testnet: ckt1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqaphtj5
  has_ckb_root: true
  lock_arg: 0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2
  lock_hash: 0xcc112ffd6128d98ee1b10f4eb9c08e3271c616d3bf2e2e1cb67ca9af94583ad8
  source: Local File System

  export private key and chain code
  0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c
  f0006f214ff8fc6266fe7fbf7baceef4f7e4116df426853263a6633493ea93ef
 */


#[derive(Deserialize, Debug)]
struct Data {
    quotes: HashMap<String, HashMap<String, f64>>
}

#[derive(Deserialize, Debug)]
struct Res {
    data: Data,
}


async fn get_price_by_cmc(url: &String) -> Result<u128, Error> {
    let ckb_resp: Res = reqwest::get(&format!("{}{}",url,"/4948/")).await?.json().await?;
    let ckb_price = ckb_resp.data.quotes.get("USD").unwrap().get("price").unwrap().clone();
    let eth_resp :Res = reqwest::get(&format!("{}{}",url,"/1027/")).await?.json().await?;
    let eth_price = eth_resp.data.quotes.get("USD").unwrap().get("price").unwrap().clone();
    let base: f64 = 10 as f64;
    // println!("ckb : {:?}, eth :{:?}", ckb,eth);
    Ok(ckb_price.mul(base.powf(18 as f64)).div(eth_price) as u128)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let request_url = String::from("https://widgets.coinmarketcap.com/v2/ticker");
    let mut input_hash = h256!("0xc26694c6d2e76e9e89899369c6756be708d00249499b207f380b355057008b99");
    let mut i: u64  = 1;
    while i < 100000 {
        // get price
        let  price = get_price_by_cmc(&request_url).await.unwrap();

        println!("price: {:?}", price.clone());

        //  CKB RPC client
        let mut rpc_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));

        let tx = get_tx(&price, CELL_CAPACITY-i*10000000, input_hash);
        input_hash = rpc_client.send_transaction(tx).map_err(|err| format!("Send transaction error: {}", err)).unwrap();
        println!("hash result i : {:?}", input_hash.clone().to_string());
        i += 1;
    }
    Ok(())
}

pub fn get_tx(price: &u128, capacity: u64, input_tx_hash: H256) -> packed::Transaction {
    let mut ckb_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));

    // let input_tx_hash = h256!("0xc610fa6abaeecbe54147683eb15f677b1ff27314230de21bc56862e62c71670f");
    let cell_input = packed::OutPoint::new_builder()
        .tx_hash(input_tx_hash.pack())
        .index(0u32.pack())
        .build();

    // cell input
    let input = packed::CellInput::new_builder()
        .previous_output(cell_input)
        .build();
    let  inputs = vec![input];

    let lock_script = gen_lockscript();

    // cell output
    let output = packed::CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script.into())
        .build();
    let  outputs = vec![output];

    // outputs_data: the price of ckb
    let  outputs_data: Vec<Bytes> = vec![Bytes::from(price.to_string())];

    let block: BlockView = ckb_client
        .get_block_by_number(0)
        .expect("get genesis block failed from ckb")
        .expect("genesis block is none")
        .into();
    let ckb_genesis_info =
        GenesisInfo::from_block(&block).expect("ckb genesisInfo generated failed");
    let secp256_dep: packed::CellDep =  ckb_genesis_info.sighash_dep();

    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(secp256_dep)
        .build();

    // import account private key for Privkey
    let private_key = "0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c";
    let mut privkey_bytes = [0u8; 32];
    hex_decode(private_key.as_bytes(), &mut privkey_bytes);

    // sign
    let privkey = ckb_crypto::secp::Privkey::from_slice(&privkey_bytes);
    let tx = sign_tx(tx, &privkey);

    tx.data()
}

pub fn sign_tx(tx: ckb_types::core::TransactionView, key: &Privkey) -> TransactionView {
    let witnesses_len = tx.witnesses().len();
    let tx_hash = tx.hash();
    let mut signed_witnesses: Vec<packed::Bytes> = Vec::new();
    let mut blake2b = new_blake2b();
    let mut message = [0u8; 32];
    blake2b.update(&tx_hash.raw_data());
    // digest the first witness
    let witness = packed::WitnessArgs::default();
    let zero_lock: Bytes = {
        let mut buf = Vec::new();
        buf.resize(SIGNATURE_SIZE, 0);
        buf.into()
    };
    let witness_for_digest = witness
        .clone()
        .as_builder()
        .lock(Some(zero_lock).pack())
        .build();
    let witness_len = witness_for_digest.as_bytes().len() as u64;
    blake2b.update(&witness_len.to_le_bytes());
    blake2b.update(&witness_for_digest.as_bytes());
    (1..witnesses_len).for_each(|n| {
        let witness = tx.witnesses().get(n).unwrap();
        let witness_len = witness.raw_data().len() as u64;
        blake2b.update(&witness_len.to_le_bytes());
        blake2b.update(&witness.raw_data());
    });
    blake2b.finalize(&mut message);
    let message = H256::from(message);
    let sig = key.sign_recoverable(&message).expect("sign");
    signed_witnesses.push(
        witness
            .as_builder()
            .lock(Some(Bytes::from(sig.serialize())).pack())
            .build()
            .as_bytes()
            .pack(),
    );
    for i in 1..witnesses_len {
        signed_witnesses.push(tx.witnesses().get(i).unwrap());
    }
    tx.as_advanced_builder()
        .set_witnesses(signed_witnesses)
        .build()
}

// input data: lock_arg
pub fn gen_lockscript() -> packed::Script {
    packed::Script::new_builder()
        .code_hash(SIGHASH_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(h160!("0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2").as_ref()).pack())
        .build()
}

