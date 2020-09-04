## 背景介绍

获取交易所 `ckb`的价格 ，通过`Transaction`提供价格到`CKB`网络中

- 获取`CKB`成交价

   从`Huobi ` 接口  `https://api.huobi.pro/market/trade?symbol=ckbusdt `  访问连接超时，暂时从`coinmarketcap`  获取

  

- 提供两种币对价格（通过换算而来）两个价格作为`output_data` (32位`Byte`)，
  -  `ckb_eth`  (以`wei`为单位换算)  前16位` ckb_wei`
  - `ckb_btc `   (以`sat`为单位换算)  后16位` ckb_sat`



## 项目运行

- 1、[本地运行`CKB` 节点](https://docs.nervos.org/docs/basics/guides/devchain)

  
  
- 2、获取`ckb`账户

  ```shell
  ➜ ckb-cli account list
  
  address:
    mainnet: ckb1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqqyf57g
    testnet: ckt1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqaphtj5 
  has_ckb_root: true
  lock_arg: 0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2
  lock_hash: 0xcc112ffd6128d98ee1b10f4eb9c08e3271c616d3bf2e2e1cb67ca9af94583ad8
  source: Local File System
  ```

  

- 3、获取 `private key `  和` chain code` 

  ```shell
  # private key and chain code
  0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c #private chain
  f0006f214ff8fc6266fe7fbf7baceef4f7e4116df426853263a6633493ea93ef # chain code
  ```

  

- 4、获取账户的有效 `cell` 信息

  ```shell
  # 获取有效的 cell作为 input
  ➜ ckb-cli wallet get-live-cells --address ckt1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqaphtj5
  
  current_capacity: 30148.26160749 (CKB)
  current_count: 15
  live_cells:
   - capacity: 2009.88396633 (CKB)
      data_bytes: 0
      index:
        output_index: 0
        tx_index: 0
      lock_hash: 0xcc112ffd6128d98ee1b10f4eb9c08e3271c616d3bf2e2e1cb67ca9af94583ad8
      mature: true
      number: 55
      output_index: 0
      tx_hash: 0x859183be46b25b24c712230da914c3944a3789ab2c4edee2cef18848b488c2fc
      type_hashes: ~
  ```

  

- 5、修改配置文件`config.toml`

  ```toml
  lock_arg="0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2"
  # 设置 私钥
  private_key="0x0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c"
  # 设置 tx_hash 作为input
  input_tx_hash="0x859183be46b25b24c712230da914c3944a3789ab2c4edee2cef18848b488c2fc"
  # 设置手续费
  fee_shannons=1000000000
  ```

  

- 6、构建运行

  ```shell
  # 构建
  ➜  price git:(master) ✗  cargo build
  # 运行
  ➜  price git:(master) ✗ ~/proxyrc.sh cargo run
      Finished dev [unoptimized + debuginfo] target(s) in 4.12s
       Running `target/debug/price`
  ckb : 0.0052919486, eth :375.271770736,  btc:10271.2045993
  price data byte:[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 211, 75, 110, 116, 199, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 51]
  hash : 29ad2e53b3dc97317d0883e7e7b5ab72ff1724c273a6621a4fa1487c5e0b51c9
  
  # 根据output hash 查询交易
  ➜  price git:(master) ✗ ckb-cli rpc get_transaction --hash 29ad2e53b3dc97317d0883e7e7b5ab72ff1724c273a6621a4fa1487c5e0b51c9 --output-format json
  {
    "transaction": {
      "cell_deps": [
        {
          "dep_type": "dep_group",
          "out_point": {
            "index": 0,
            "tx_hash": "0xace5ea83c478bb866edf122ff862085789158f5cbff155b7bb5f13058555b708"
          }
        }
      ],
      "hash": "0x29ad2e53b3dc97317d0883e7e7b5ab72ff1724c273a6621a4fa1487c5e0b51c9",
      "header_deps": [],
      "inputs": [
        {
          "previous_output": {
            "index": 0,
            "tx_hash": "0x859183be46b25b24c712230da914c3944a3789ab2c4edee2cef18848b488c2fc"
          },
          "since": "0x0 (absolute block(0))"
        }
      ],
      "outputs": [
        {
          "capacity": "190.0",
          "lock": {
            "args": "0x4ddd5d32e1ee8bed83360fac2f70b03a6e4378b2",
            "code_hash": "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8 (sighash)",
            "hash_type": "type"
          },
          "type": null
        }
      ],
      "outputs_data": [ # 两个price，前16位 ckb_wei 后16位 ckb_sat
        "0x000000000000000000000cd34b6e74c700000000000000000000000000000033"
      ],
      "version": 0,
      "witnesses": [
        "0x5500000010000000550000005500000041000000f404cf1d599705a735ec4ca777731381d0778a4f5a3cb34c84e0c46e51aeb8f7413c9fc2c61d8986e607b21a3bae6007e70eb8d1838fc1cc3c63f4113e71856601"
      ]
    },
    "tx_status": {
      "block_hash": "0x5a50a4858e7401cf0119e9f8f4606dd22c34546697bb6b634298481fabb7c83e",
      "status": "committed"
    }
  }
  ```

   

  

