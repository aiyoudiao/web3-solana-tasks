## Typescript

铸造 SPL 代币


https://learn.blueshift.gg/zh-CN/challenges/typescript-mint-an-spl-token/verify

## secret 使用

可以通过命令 pnpm g 获取，可以直接在运行时 pnpm mint 边获取变应用。

## 查看

### 本地

1. 先启动本地测试环境
```
solana-test-validator --reset
solana config set --url http://127.0.0.1:8899
solana airdrop 10
solana balance
```

2.修改 `.env` 文件

```
RPC_ENDPOINT="http://127.0.0.1:8899"
```

3. 运行

```
pnpm mint
```

4. 查看结果
https://explorer.solana.com/address/6u5q6ZNCNwZL9wyTxPHf94ZfC4fiTsq59ks6m2ssbpea?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899

### 第三方 devnet 开发环境

1.  https://www.helius.dev/ 注册账号，获取 devnet 的 RPC_ENDPOINT

2. 先配置本地环境
```
solana config set --url https://devnet.helius-rpc.com/?api-key={api-key}
solana balance
```

3. 修改 `.env` 文件

```
RPC_ENDPOINT="https://devnet.helius-rpc.com/?api-key={api-key}"
```

4. 如果没有 SOLE ，请使用 https://faucet.solana.com/ 领取 SOL

```
# 获取本地钱包地址
solana address
```


5. 运行项目

如果老是失败，大概是你的网络不稳定，请重新运行

```
pnpm mint
```

上面不行可以尝试用 pnpm temp-mint(临时不去管 HTTPS 验证书)

1. 查看结果

https://explorer.solana.com/address/3CALF2fMyaUxMVosUJnRk55DbXxou9fKBZM86ZPi9Wde?cluster=devnet
