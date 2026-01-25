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
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>, // 交易接受者（Taker）

    #[account(mut)]
    pub maker: SystemAccount<'info>, // 原始创建者（Maker），用于接收代币B

    #[account(
        mut,
        close = maker, // 交易完成后关闭 Escrow 账户，租金退还给 Maker
        has_one = maker @ EscrowError::InvalidMaker, // 验证 Escrow 的 maker 字段匹配
        has_one = mint_a @ EscrowError::InvalidMintA, // 验证 mint_a 匹配
        has_one = mint_b @ EscrowError::InvalidMintB, // 验证 mint_b 匹配
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()], // 验证 PDA 种子
        bump = escrow.bump, // 验证 bump
    )]
    pub escrow: Box<Account<'info, Escrow>>, // 托管账户

    #[account(mint::token_program = token_program)]
    pub mint_a: Box<InterfaceAccount<'info, Mint>>, // 代币A（Maker提供的）

    #[account(mint::token_program = token_program)]
    pub mint_b: Box<InterfaceAccount<'info, Mint>>, // 代币B（Maker想要的）

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>, // 存放代币A的金库

    #[account(
        init_if_needed, // 如果 Taker 没有代币A账户，则初始化
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>, // Taker 接收代币A的账户

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>, // Taker 支付代币B的账户

    #[account(
        init_if_needed, // 如果 Maker 没有代币B账户，则初始化
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>, // Maker 接收代币B的账户

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    // 将代币B从 Taker 转移到 Maker
    pub fn transfer_to_maker(&mut self) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_b.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                    to: self.maker_ata_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.receive,
            self.mint_b.decimals,
        )?;
        Ok(())
    }

    // 将代币A从 Vault 转移到 Taker，并关闭 Vault
    pub fn withdraw_and_close_vault(&mut self) -> Result<()> {
        // 准备 PDA 签名种子
        let signer_seeds: [&[&[u8]]; 1] = [&[
            ESCROW_SEED,
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // 执行代币A转账
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.taker_ata_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        // 关闭金库账户
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                destination: self.maker.to_account_info(), // 剩余 SOL 退还给 Maker
                authority: self.escrow.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Take>) -> Result<()> {
    msg!("开始执行交易...");
    // 1. 将代币B从 Taker 转移到 Maker
    ctx.accounts.transfer_to_maker()?;
    msg!("已将 {} 个代币B转移给 Maker", ctx.accounts.escrow.receive);

    // 2. 将代币A从 Vault 转移到 Taker，并关闭金库
    ctx.accounts.withdraw_and_close_vault()?;
    msg!("已将代币A转移给 Taker，并关闭了金库账户");

    Ok(())
}
