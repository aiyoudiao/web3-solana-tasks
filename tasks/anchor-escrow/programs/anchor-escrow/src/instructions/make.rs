use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    errors::EscrowError,
    state::{Escrow, ESCROW_SEED},
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>, // 交易发起人（创建者）

    #[account(
        init,
        payer = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
    )]
    pub escrow: Account<'info, Escrow>, // 托管账户 PDA

    #[account(mint::token_program = token_program)]
    pub mint_a: InterfaceAccount<'info, Mint>, // 代币A的铸造账户（Maker存入的）

    #[account(mint::token_program = token_program)]
    pub mint_b: InterfaceAccount<'info, Mint>, // 代币B的铸造账户（Maker想要的）

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>, // Maker持有代币A的账户

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>, // 存放代币A的金库（由Escrow PDA控制）

    pub associated_token_program: Program<'info, AssociatedToken>, // 关联代币程序
    pub token_program: Interface<'info, TokenInterface>, // 代币程序
    pub system_program: Program<'info, System>, // 系统程序
}

impl<'info> Make<'info> {
    // 填充 Escrow 账户数据
    pub fn populate_escrow(&mut self, seed: u64, receive: u64, bump: u8) -> Result<()> {
        self.escrow.seed = seed;
        self.escrow.maker = self.maker.key();
        self.escrow.mint_a = self.mint_a.key();
        self.escrow.mint_b = self.mint_b.key();
        self.escrow.receive = receive;
        self.escrow.bump = bump;
        Ok(())
    }

    // 将代币存入金库
    pub fn deposit_tokens(&mut self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
    msg!("开始初始化托管账户...");
    require_gt!(receive, 0, EscrowError::InvalidAmount);
    require_gt!(deposit, 0, EscrowError::InvalidAmount);

    // 保存状态到 Escrow 账户
    ctx.accounts.populate_escrow(seed, receive, ctx.bumps.escrow)?;
    msg!("托管账户状态已保存: receive={}, seed={}", receive, seed);

    // 将代币A从 Maker 转移到 Vault
    ctx.accounts.deposit_tokens(deposit)?;
    msg!("已将 {} 个代币A存入金库", deposit);

    Ok(())
}
