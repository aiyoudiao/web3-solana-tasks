import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorEscrow } from "../target/types/anchor_escrow";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

describe("anchor-escrow", () => {
  // 配置客户端使用本地集群
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AnchorEscrow as Program<AnchorEscrow>;
  const provider = anchor.getProvider();

  const maker = Keypair.generate();
  const taker = Keypair.generate();

  let mintA: PublicKey;
  let mintB: PublicKey;

  let makerAtaA: PublicKey;
  let makerAtaB: PublicKey;

  let takerAtaA: PublicKey;
  let takerAtaB: PublicKey;

  const seed = new BN(1); // 托管种子
  const depositAmount = new BN(100);
  const receiveAmount = new BN(200);

  // 托管账户的 PDA
  const [escrowPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      maker.publicKey.toBuffer(),
      seed.toArrayLike(Buffer, "le", 8),
    ],
    program.programId,
  );

  before(async () => {
    // 给 maker 和 taker 空投 SOL
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        maker.publicKey,
        10 * LAMPORTS_PER_SOL,
      ),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        taker.publicKey,
        10 * LAMPORTS_PER_SOL,
      ),
    );

    // 创建代币铸造账户 (Mints)
    mintA = await createMint(
      provider.connection,
      maker,
      maker.publicKey,
      null,
      6,
    );
    mintB = await createMint(
      provider.connection,
      taker,
      taker.publicKey,
      null,
      6,
    );

    // 创建关联代币账户 (ATAs)
    makerAtaA = await createAssociatedTokenAccount(
      provider.connection,
      maker,
      mintA,
      maker.publicKey,
    );
    makerAtaB = await createAssociatedTokenAccount(
      provider.connection,
      maker,
      mintB,
      maker.publicKey,
    ); // 稍后用于接收代币B

    takerAtaA = await createAssociatedTokenAccount(
      provider.connection,
      taker,
      mintA,
      taker.publicKey,
    ); // 稍后用于接收代币A
    takerAtaB = await createAssociatedTokenAccount(
      provider.connection,
      taker,
      mintB,
      taker.publicKey,
    );

    // 铸造代币
    await mintTo(provider.connection, maker, mintA, makerAtaA, maker, 1000);
    await mintTo(provider.connection, taker, mintB, takerAtaB, taker, 1000);
  });

  it("挑战 1：创建托管报价 (Make)", async () => {
    // 金库 ATA (由托管 PDA 所有)
    const vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

    console.log("正在创建托管，种子为:", seed.toString());

    const tx = await program.methods
      .make(seed, depositAmount, receiveAmount)
      .accounts({
        maker: maker.publicKey,
        escrow: escrowPda,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    console.log("Make 交易签名", tx);

    // 验证托管状态
    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    assert.ok(escrowAccount.maker.equals(maker.publicKey));
    assert.ok(escrowAccount.mintA.equals(mintA));
    assert.ok(escrowAccount.mintB.equals(mintB));
    assert.ok(escrowAccount.receive.eq(receiveAmount));

    // 验证金库余额
    const vaultAccount = await getAccount(provider.connection, vault);
    assert.equal(Number(vaultAccount.amount), depositAmount.toNumber());
  });

  it("挑战 2：接受托管报价 (Take)", async () => {
    const vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);

    console.log("正在接受托管...");

    // Taker 接受报价
    const tx = await program.methods
      .take()
      .accounts({
        taker: taker.publicKey,
        maker: maker.publicKey,
        escrow: escrowPda,
        mintA: mintA,
        mintB: mintB,
        vault: vault,
        takerAtaA: takerAtaA,
        takerAtaB: takerAtaB,
        makerAtaB: makerAtaB,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([taker])
      .rpc();

    console.log("Take 交易签名", tx);

    // 验证余额
    const takerAAccount = await getAccount(provider.connection, takerAtaA);
    const makerBAccount = await getAccount(provider.connection, makerAtaB);

    assert.equal(Number(takerAAccount.amount), depositAmount.toNumber()); // Taker 获得了 A
    assert.equal(Number(makerBAccount.amount), receiveAmount.toNumber()); // Maker 获得了 B

    // 验证托管账户已关闭 (应该抛出错误)
    try {
      await program.account.escrow.fetch(escrowPda);
      assert.fail("托管账户应该已被关闭");
    } catch (e: any) {
      assert.ok(
        e.message.includes("Account does not exist") ||
          e.message.includes("Could not find account"),
      );
    }
  });

  it("挑战 3：取消托管报价 (Refund)", async () => {
    // 为退款测试设置一个新的托管
    const seedRefund = new BN(2);
    const [escrowPdaRefund] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        seedRefund.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    );
    const vaultRefund = getAssociatedTokenAddressSync(
      mintA,
      escrowPdaRefund,
      true,
    );

    console.log("正在为退款测试创建新托管...");

    // 再次创建托管 (Make)
    await program.methods
      .make(seedRefund, depositAmount, receiveAmount)
      .accounts({
        maker: maker.publicKey,
        escrow: escrowPdaRefund,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        vault: vaultRefund,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    // 退款前验证金库余额
    let vaultAccount = await getAccount(provider.connection, vaultRefund);
    assert.equal(Number(vaultAccount.amount), depositAmount.toNumber());

    console.log("正在退款托管...");

    // 退款 (Refund)
    const tx = await program.methods
      .refund()
      .accounts({
        maker: maker.publicKey,
        escrow: escrowPdaRefund,
        mintA: mintA,
        vault: vaultRefund,
        makerAtaA: makerAtaA,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    console.log("Refund 交易签名", tx);

    // 验证托管账户已关闭
    try {
      await program.account.escrow.fetch(escrowPdaRefund);
      assert.fail("托管账户应该已被关闭");
    } catch (e: any) {
      assert.ok(
        e.message.includes("Account does not exist") ||
          e.message.includes("Could not find account"),
      );
    }

    // 验证 Maker 收回了资金
    // 初始: 1000
    // 测试 1 Make: -100 -> 900
    // 测试 2 Take: 900 (Taker 获得 A)
    // 测试 3 Make: -100 -> 800
    // 测试 3 Refund: +100 -> 900
    const makerAAccount = await getAccount(provider.connection, makerAtaA);
    assert.equal(Number(makerAAccount.amount), 900);
  });
});
