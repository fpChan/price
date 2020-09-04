use ckb_types::{H160, H256};
use config::{File, Config, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PriceConfig {
    pub lock_arg: H160,
    pub private_key: H256,
    pub input_tx_hash: H256,
    pub fee_shannons : u64,
}

pub fn load_price_config() -> PriceConfig {
    // Create a new local configuration
    let mut c = Config::new();

    c.merge(File::new("config", FileFormat::Toml).required(false)).unwrap();
    // c.try_into::<PriceConfig>().unwrap()

    c.try_into()
        .map_err(|e| panic!("load price_config error: {}", e))
        .unwrap()
}
