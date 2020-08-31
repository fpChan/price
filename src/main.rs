use std::collections::{HashMap};
use reqwest;
use serde::Deserialize;
use reqwest::Error;

use ckb_sdk::{
    constants::{SIGHASH_TYPE_HASH},
    wallet::DerivationPath, HttpRpcClient, SignerFn, TxHelper,
};

use ckb_types::{
    bytes::Bytes,
    core::{ScriptHashType, TransactionBuilder, TransactionView},
    h160, h256,
    packed,
    prelude::*,
    H160, H256,
};

use ckb_crypto::secp::Privkey;
use ckb_hash::{blake2b_256, new_blake2b};


use faster_hex::{hex_decode};


const SIGNATURE_SIZE: usize = 65;
const CELL_CAPACITY: u64 = 16 * 100000000 + 14100000000;

#[derive(Deserialize, Debug)]
struct Data {
    quotes: HashMap<String, HashMap<String, f64>>
}

#[derive(Deserialize, Debug)]
struct Res {
    data: Data,
}

#[tokio::main]
async fn main() -> Result<(), Error> {

    // get price
    let request_url = String::from("https://widgets.coinmarketcap.com/v2/ticker/4948/");
    let response: Res = reqwest::get(&request_url).await?.json().await?;
    let price = response.data.quotes.get("USD").unwrap().get("price").unwrap();
    println!("price: {:?}", price);
    //  CKB RPC client
    let mut rpc_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));

    // let mut helper = TxHelper::default();

    let tx = get_tx(price);
    let tx_hash = rpc_client.send_transaction(tx).map_err(|err| format!("Send transaction error: {}", err)).unwrap();
    println!("{:?}",tx_hash);

    Ok(())
}


pub fn get_tx(price:&f64) -> packed::Transaction {

    let input_tx_hash = h256!("0x22048803dae600f56ffb95a340788d3131676005d8bdd0546653b338afd7e785");

    let cell_input = packed::OutPoint::new_builder()
        .tx_hash(input_tx_hash.pack())
        .index(0u32.pack())
        .build();

    // cell input
    let input = packed::CellInput::new_builder()
        .previous_output(cell_input.clone())
        .build();
    let mut inputs = vec![input];


    let output_lock_code_hash = "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";
    let mut output_lockscript_bytes = [0u8; 20];
    hex_decode(&output_lock_code_hash.as_bytes()[2..], &mut output_lockscript_bytes)
        .expect("hex decode privkey_hex error");
    let output_lockscript = gen_lockscript(std::convert::From::from(output_lockscript_bytes));

    // cell output
    let output = packed::CellOutput::new_builder()
        .capacity(CELL_CAPACITY.pack())
        .lock(output_lockscript.into())
        .build();
    let mut outputs = vec![output];

    // outputs_data: the price of ckb
    let mut outputs_data: Vec<Bytes> = vec![Bytes::from(price.to_string())];


    // build transaction
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();

    // import private key for Privkey
    let private_key = "0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c";
    let mut privkey_bytes = [0u8; 32];
    hex_decode(private_key.as_bytes(), &mut privkey_bytes);

    // sign
    let privkey = ckb_crypto::secp::Privkey::from_slice(&privkey_bytes);
    let tx = sign_tx(tx, &privkey);
    println!("data sign tx:  {}",tx.clone() );

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


pub fn gen_lockscript(lock_args: H160) -> packed::Script {
    packed::Script::new_builder()
        .code_hash(SIGHASH_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(lock_args.as_bytes().to_vec()).pack())
        .build()
}