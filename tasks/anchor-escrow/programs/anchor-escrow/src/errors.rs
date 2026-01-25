use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("金额无效")]
    InvalidAmount,
    #[msg("创建者无效")]
    InvalidMaker,
    #[msg("代币A铸造地址无效")]
    InvalidMintA,
    #[msg("代币B铸造地址无效")]
    InvalidMintB,
}
