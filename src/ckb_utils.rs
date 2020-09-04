use ckb_sdk::{
    constants::{SIGHASH_TYPE_HASH},
    HttpRpcClient,GenesisInfo,
};
use ckb_types::core::BlockView;

use ckb_types::{
    bytes::Bytes,
    core::{ScriptHashType, TransactionBuilder, TransactionView},
    packed,
    prelude::*,
    H160, H256,
};

use ckb_crypto::secp::{Privkey, SECP256K1};
use ckb_hash::{new_blake2b,blake2b_256};

const SIGNATURE_SIZE: usize = 65;


pub fn send_tx(capacity: u64, input_tx_hash: H256, private_key:H256, prices: (u128,u128)) -> Result<H256, String> {
    let mut ckb_client = HttpRpcClient::new(String::from("http://127.0.0.1:8114"));

    let cell_input = packed::OutPoint::new_builder()
        .tx_hash(input_tx_hash.pack())
        .index(0u32.pack())
        .build();

    // cell input
    let input = packed::CellInput::new_builder()
        .previous_output(cell_input)
        .build();
    let  inputs = vec![input];

    let lock_script = gen_lockscript(gen_lock_args(private_key.clone()));

    // cell output
    let output = packed::CellOutput::new_builder()
        .capacity(capacity.pack())
        .lock(lock_script.into())
        .build();
    let  outputs = vec![output];


    let output_data  = merge_u128(prices.0,prices.1);

    println!("price data byte:{:?}", output_data.clone());

    let  outputs_data: Vec<Bytes> = vec![Bytes::from(output_data)];

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

    // sign
    let privkey = ckb_crypto::secp::Privkey::from_slice(private_key.as_bytes());
    let tx = sign_tx(tx, &privkey);

    ckb_client.send_transaction(tx.data())

}


pub fn gen_lock_args(privkey_key: H256) -> H160 {
    let privkey = secp256k1::SecretKey::from_slice(privkey_key.as_bytes()).unwrap();
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &privkey);

    let lock_arg = H160::from_slice(&blake2b_256(&pubkey.serialize()[..])[0..20])
        .expect("Generate hash(H160) from pubkey failed");
    lock_arg
}


pub fn gen_lockscript(lock_args: H160) -> packed::Script {
    packed::Script::new_builder()
        .code_hash(SIGHASH_TYPE_HASH.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(lock_args.as_bytes().to_vec()).pack())
        .build()
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

fn merge_u128(first: u128, second: u128) -> Vec<u8> {

    let mut  result = Vec::new();
    let d1  = first.to_be_bytes();
    let d2  = second.to_be_bytes();
    for &i in d1.iter() {
        result.push(i)
    }
    for &i in d2.iter() {
        result.push(i)
    }

    result
}
