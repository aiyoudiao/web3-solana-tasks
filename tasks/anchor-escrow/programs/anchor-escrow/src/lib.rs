use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;

use instructions::*;

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod anchor_escrow {
    use super::*;

    // 创建托管
    // discriminator = 0 指定指令鉴别器
    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        msg!("调用 Make 指令: seed={}, deposit={}, receive={}", seed, deposit, receive);
        instructions::make::handler(ctx, seed, deposit, receive)
    }

    // 接受托管
    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        msg!("调用 Take 指令");
        instructions::take::handler(ctx)
    }

    // 退款
    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        msg!("调用 Refund 指令");
        instructions::refund::handler(ctx)
    }
}
