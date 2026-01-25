use anchor_lang::prelude::*;

// 托管账户的种子常量
pub const ESCROW_SEED: &[u8] = b"escrow";

#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct Escrow {
    pub seed: u64,           // 种子，允许同一用户创建多个托管
    pub maker: Pubkey,       // 创建者的公钥
    pub mint_a: Pubkey,      // 代币A的铸造地址（Maker提供的代币）
    pub mint_b: Pubkey,      // 代币B的铸造地址（Maker想要的代币）
    pub receive: u64,        // Maker希望收到的代币B的数量
    pub bump: u8,            // PDA的bump值，用于后续签名
}
