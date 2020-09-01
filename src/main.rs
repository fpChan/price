use std::collections::{HashMap};
use reqwest;
use serde::Deserialize;
use reqwest::Error;

use ckb_sdk::{
    constants::{SIGHASH_TYPE_HASH},
    HttpRpcClient,
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

use ckb_crypto::secp::{Privkey, SECP256K1};
use ckb_hash::{blake2b_256, new_blake2b};


use faster_hex::{hex_decode};
use ckb_types::packed::{CellDep, OutPoint, CellOutput, Byte32};
use ckb_types::core::DepType;


const SIGNATURE_SIZE: usize = 65;
const CELL_CAPACITY: u64 = 16 * 100000000 + 14000000000;
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let request_url = String::from("https://widgets.coinmarketcap.com/v2/ticker/4948/");
    let mut input_hash = "0x6f2321befe738192959de2f64e6827b2c3afc978ae4889ce5383776fe87f4cae";

    // get price
    let response: Res = reqwest::get(&request_url).await?.json().await?;
    let price = response.data.quotes.get("USD").unwrap().get("price").unwrap();
    println!("price: {:?}", price);

    //  CKB RPC client
    let mut rpc_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));

    let tx = get_tx(price, input_hash.to_owned().clone());
    let tx_hash = rpc_client.send_transaction(tx).map_err(|err| format!("Send transaction error: {}", err)).unwrap();
    println!("hash result 1 : {:?}", tx_hash.to_string());


    Ok(())
}

pub fn get_tx(price: &f64, input_tx_hash: String) -> packed::Transaction {
    let mut ckb_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));


    println!("the parpms : {}", input_tx_hash.clone());
    // let input_tx_hash = h256!("0x6f2321befe738192959de2f64e6827b2c3afc978ae4889ce5383776fe87f4cae");
    let tx_hash = input_tx_hash.clone();

    let mut tx_hash_bytes = [0u8; 32];
    hex_decode(&tx_hash.as_bytes(), &mut tx_hash_bytes);


    let cell_input = packed::OutPoint::new_builder()
        .tx_hash(H256::from(tx_hash_bytes).pack())
        .index(0u32.pack())
        .build();

    // cell input
    let input = packed::CellInput::new_builder()
        .previous_output(cell_input.clone())
        .build();
    let mut inputs = vec![input];

    let lock_script = gen_lockscript();

    // cell output
    let output = packed::CellOutput::new_builder()
        .capacity(CELL_CAPACITY.pack())
        .lock(lock_script.into())
        .build();
    let mut outputs = vec![output];

    // outputs_data: the price of ckb
    let mut outputs_data: Vec<Bytes> = vec![Bytes::from(price.to_string())];

    // let block: BlockView = ckb_client
    //     .get_block_by_number(0)
    //     .expect("get genesis block failed from ckb")
    //     .expect("genesis block is none")
    //     .into();
    // let ckb_genesis_info =
    //     GenesisInfo::from_block(&block).expect("ckb genesisInfo generated failed");
    // let secp256_dep: packed::CellDep =  ckb_genesis_info.sighash_dep();

    let secp256_dep: packed::CellDep = CellDep::new_builder()
        .out_point(cell_input.clone())
        .dep_type(DepType::DepGroup.into())
        .build();
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
        .hash_type(ScriptHashType::Data.into())
        .args(Bytes::from(h160!("0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2").as_ref()).pack())
        .build()
}