use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{
    errors::EscrowError,
    state::{Escrow, ESCROW_SEED},
};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // 原始创建者（Maker），发起退款

    #[account(
        mut,
        close = maker, // 关闭 Escrow 账户，租金退还给 Maker
        has_one = maker @ EscrowError::InvalidMaker, // 验证 maker
        has_one = mint_a @ EscrowError::InvalidMintA, // 验证 mint_a
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()], // 验证 PDA 种子
        bump = escrow.bump, // 验证 bump
    )]
    pub escrow: Account<'info, Escrow>, // 托管账户

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>, // 代币A

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // 金库账户

    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>, // Maker 接收退回代币A的账户

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Refund>) -> Result<()> {
    msg!("开始执行退款...");
    let vault_amount = ctx.accounts.vault.amount;
    let escrow = &ctx.accounts.escrow;
    let seed_bytes = escrow.seed.to_le_bytes();
    
    // 准备 PDA 签名种子
    let signer_seeds: &[&[u8]] = &[
        ESCROW_SEED,
        escrow.maker.as_ref(),
        seed_bytes.as_ref(),
        &[escrow.bump],
    ];
    let signer = &[signer_seeds];

    // 1. 如果金库中有余额，将所有代币A退还给 Maker
    if vault_amount > 0 {
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.mint_a.to_account_info(),
                    to: ctx.accounts.maker_ata_a.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                signer,
            ),
            vault_amount,
            ctx.accounts.mint_a.decimals,
        )?;
        msg!("已将 {} 个代币A退还给 Maker", vault_amount);
    }

    // 2. 关闭金库账户
    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(), // 剩余 SOL 退还给 Maker
            authority: ctx.accounts.escrow.to_account_info(),
        },
        signer,
    ))?;
    msg!("金库账户已关闭");

    Ok(())
}
