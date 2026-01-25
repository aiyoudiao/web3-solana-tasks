import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BlueshiftAnchorVault } from "../target/types/blueshift_anchor_vault";
import { assert } from "chai";

describe("blueshift_anchor_vault", () => {
  // 配置客户端使用本地集群
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .BlueshiftAnchorVault as Program<BlueshiftAnchorVault>;
  const signer = provider.wallet as anchor.Wallet;

  // 派生金库 PDA
  const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), signer.publicKey.toBuffer()],
    program.programId,
  );

  it("Deposits SOL into the vault", async () => {
    // 存入 1 SOL (1_000_000_000 lamports)
    const amount = new anchor.BN(1_000_000_000);

    // 获取存款前的余额
    const preSignerBalance = await provider.connection.getBalance(
      signer.publicKey,
    );

    console.log(`\n当前余额: ${preSignerBalance} SOL`);

    console.log("Depositing 1 SOL...");

    try {
      const tx = await program.methods
        .deposit(amount)
        .accounts({
          signer: signer.publicKey,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Deposit Transaction signature", tx);

      // 验证金库余额
      const vaultBalance = await provider.connection.getBalance(vaultPda);
      console.log("Vault Balance:", vaultBalance);
      assert.equal(vaultBalance, amount.toNumber(), "Vault should have 1 SOL");
    } catch (e) {
      console.error(e);
      throw e;
    }
  });

  it("Withdraws SOL from the vault", async () => {
    console.log("Withdrawing all SOL...");

    try {
      const tx = await program.methods
        .withdraw()
        .accounts({
          signer: signer.publicKey,
          vault: vaultPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Withdraw Transaction signature", tx);

      // 验证金库余额应为 0
      const vaultBalance = await provider.connection.getBalance(vaultPda);
      console.log("Vault Balance after withdraw:", vaultBalance);
      assert.equal(vaultBalance, 0, "Vault should be empty");
    } catch (e) {
      console.error(e);
      throw e;
    }
  });
});
