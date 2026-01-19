/** Challenge: Mint an SPL Token
 *
 * In this challenge, you will create an SPL token!
 *
 * Goal:
 *   Mint an SPL token in a single transaction using Web3.js and the SPL Token library.
 *
 * Objectives:
 *   1. Create an SPL mint account.
 *   2. Initialize the mint with 6 decimals and your public key (feePayer) as the mint and freeze authorities.
 *   3. Create an associated token account for your public key (feePayer) to hold the minted tokens.
 *   4. Mint 21,000,000 tokens to your associated token account.
 *   5. Sign and send the transaction.
 */

import {
  Keypair,
  Connection,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

import {
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  createMintToCheckedInstruction,
  MINT_SIZE,
  getMinimumBalanceForRentExemptMint,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,

  ASSOCIATED_TOKEN_PROGRAM_ID
} from "@solana/spl-token";

import bs58 from "bs58";

// Import our keypair from the wallet file
const feePayer = Keypair.fromSecretKey(
  // ⚠️ INSECURE KEY. DO NOT USE OUTSIDE OF THIS CHALLENGE
  bs58.decode(process.env.SECRET)
);

//Create a connection to the RPC endpoint
const connection = new Connection(
  process.env.RPC_ENDPOINT,
  "confirmed"
);

// Entry point of your TypeScript code (we will call this)
async function main() {
  try {

    // Generate a new keypair for the mint account
    const mint = Keypair.generate();

    const mintRent = await getMinimumBalanceForRentExemptMint(connection);

    // START HERE


    // 创建 mint 账户的指令：由 feePayer 支付租金，在 TOKEN_PROGRAM_ID 名下创建 mint 账户
    const createAccountIx = SystemProgram.createAccount({
      fromPubkey: feePayer.publicKey, // 付钱并创建账户的钱包（也是交易费支付者）
      newAccountPubkey: mint.publicKey, // 要创建的新账户公钥（用来作为 mint）
      lamports: mintRent, // 拥有的 lamports，保证 mint 账户免租金
      space: MINT_SIZE, // mint 账户需要的字节大小（由 SPL 库给出）
      programId: TOKEN_PROGRAM_ID, // mint 账户的 owner 设置为 SPL Token 程序
    });

    // 初始化 mint 账户：
    //   - 设置小数位数为 6
    //   - mintAuthority 和 freezeAuthority 都设为 feePayer 自己
    const initializeMintIx = createInitializeMint2Instruction(
      mint.publicKey, // 要初始化的 mint 账户
      6, // 代币小数位数（6 位）
      feePayer.publicKey, // mint 权限（谁可以增发）
      feePayer.publicKey // freeze 权限（谁可以冻结账户，这里也设为自己）
    );

    // 计算 feePayer 对应的关联代币账户地址（ATA），用来实际持有这个新代币
    const associatedTokenAccount = getAssociatedTokenAddressSync(
      mint.publicKey, // 代币的 mint 地址
      feePayer.publicKey // 代币账户的 owner（也就是你自己）
    );

    // 创建关联代币账户的指令：由 feePayer 支付租金，为自己创建持有新代币的 ATA
    const createAssociatedTokenAccountIx =
      createAssociatedTokenAccountInstruction(
        feePayer.publicKey, // 支付创建 ATA 所需费用的钱包
        associatedTokenAccount, // 要创建的关联代币账户地址
        feePayer.publicKey, // 该代币账户的所有者
        mint.publicKey // 该代币账户对应的 mint
      );

    // 准备要铸造的数量：21,000,000 枚代币，精度为 6 位小数
    // 实际链上存储的整数 = 代币数量 * 10^decimals
    const mintAmount = 21_000_000n * 1_000_000n; // 21,000,000 * 10^6 = 21,000,000,000,000

    // 铸造指令（带精度检查）：把 mintAmount 个“最小单位”代币铸造到 ATA 中
    const mintToCheckedIx = createMintToCheckedInstruction(
      mint.publicKey, // 要从哪个 mint 铸造
      associatedTokenAccount, // 把代币铸造到哪个账户（ATA）
      feePayer.publicKey, // 具有 mint 权限的地址（这里是 feePayer）
      mintAmount, // 铸造的数量（已经乘以 10^decimals）
      6 // 小数位数，供合约做校验
    );


    const recentBlockhash = await connection.getLatestBlockhash();

    const transaction = new Transaction({
      feePayer: feePayer.publicKey,
      blockhash: recentBlockhash.blockhash,
      lastValidBlockHeight: recentBlockhash.lastValidBlockHeight
    }).add(
        createAccountIx,
        initializeMintIx,
        createAssociatedTokenAccountIx,
        mintToCheckedIx
    );

    const transactionSignature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [feePayer, mint]  // This is the list of signers. Who should be signing this transaction?
    );

    console.log("Mint Address:", mint.publicKey.toBase58());
    console.log("Transaction Signature:", transactionSignature);
  } catch (error) {
    console.error(`Oops, something went wrong: ${error}`);
  }
}
