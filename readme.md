## 获取CKB成交价

从`Huobi ` 接口  `https://api.huobi.pro/market/trade?symbol=ckbusdt `  访问连接超时，暂时从`coinmarketcap`  获取价格



## 设置CKB账户信息

- 1、获取账户

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

  

- 2、获取 `private key `  和` chain code` 

  ```shell
  # private key and chain code
  0b008309faea434378cdfcdb36f3c21e477406af29d224edfcb3576ce348f56c
  f0006f214ff8fc6266fe7fbf7baceef4f7e4116df426853263a6633493ea93ef
  ```

  

- 3、获取账户的有效 `cell` 信息

  ```shell
  # 获取有效的 cell作为 input
  ➜ ckb-cli wallet get-live-cells --address ckt1qyqymh2axts7azldsvmqltp0wzcr5mjr0zeqaphtj5
  
  current_capacity: 30148.26160749 (CKB)
  current_count: 15
  live_cells:
    - capacity: 2009.88420817 (CKB)
      data_bytes: 0
      index:
        output_index: 0
        tx_index: 0
      lock_hash: 0xcc112ffd6128d98ee1b10f4eb9c08e3271c616d3bf2e2e1cb67ca9af94583ad8
      mature: true
      number: 15
      output_index: 0
      tx_hash: 0x33255075bc40c62ae0764c7a98ab2db1b308d8bab993a9d3fab79c36ee27242a
      type_hashes: ~
  ```

  

- 4、将 `    tx_hash: 0x33255075bc40c62ae0764c7a98ab2db1b308d8bab993a9d3fab79c36ee27242a`  作为`input`