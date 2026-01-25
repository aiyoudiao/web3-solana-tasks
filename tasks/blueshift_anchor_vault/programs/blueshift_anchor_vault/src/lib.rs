use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

// 声明程序 ID，这是 Anchor 用来识别本程序的唯一标识符。
// 在本作业中，我们使用特定的测试 ID：22222222222222222222222222222222222222222222
declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod blueshift_anchor_vault {
    use super::*;

    /// 存款指令：将 lamports 从签名者账户转移到金库（Vault）账户。
    /// 
    /// 参数:
    /// - ctx: 包含了执行该指令所需的账户上下文 (VaultAction)。
    /// - amount: 要存入的 lamports 数量 (u64 类型)。
    pub fn deposit(ctx: Context<VaultAction>, amount: u64) -> Result<()> {
        // 1. 检查金库是否为空。
        // 为了防止重复初始化或状态覆盖，我们要求金库当前的余额（lamports）必须为 0。
        // 如果不为 0，则抛出 VaultAlreadyExists 错误。
        require_eq!(ctx.accounts.vault.lamports(), 0, VaultError::VaultAlreadyExists);

        // 2. 检查存款金额是否有效。
        // Solana 上的账户需要维持一定的最低余额以豁免租金（Rent Exempt）。
        // 我们确保存入的金额大于系统账户创建所需的最低豁免租金余额。
        // Rent::get()? 获取当前的租金配置。
        // minimum_balance(0) 计算数据长度为 0 的账户所需的最低余额。
        require_gt!(amount, Rent::get()?.minimum_balance(0), VaultError::InvalidAmount);

        // 3. 执行跨程序调用 (CPI) 进行转账。
        // 我们调用系统程序 (System Program) 的 transfer 指令，将 SOL 从 signer 转移到 vault。
        
        // 构建 CPI 上下文
        let cpi_context = CpiContext::new(
            // 目标程序：系统程序
            ctx.accounts.system_program.to_account_info(),
            // 转账参数结构体
            Transfer {
                from: ctx.accounts.signer.to_account_info(), // 资金来源：签名者
                to: ctx.accounts.vault.to_account_info(),    // 资金去向：金库 PDA
            },
        );

        // 调用 anchor_lang 提供的 system_program::transfer 辅助函数执行转账
        transfer(cpi_context, amount)?;

        Ok(())
    }

    /// 取款指令：将金库中的所有 lamports 转回给签名者。
    /// 
    /// 参数:
    /// - ctx: 包含了执行该指令所需的账户上下文 (VaultAction)。
    pub fn withdraw(ctx: Context<VaultAction>) -> Result<()> {
        // 1. 检查金库是否为空。
        // 如果金库里没有钱，就无法取款，抛出 InvalidAmount 错误。
        // require_neq! 宏确保两个值不相等。
        require_neq!(ctx.accounts.vault.lamports(), 0, VaultError::InvalidAmount);

        // 2. 获取 PDA 的签名种子。
        // 因为 vault 是一个 PDA (程序派生地址)，它没有私钥。
        // 要从 PDA 转账，程序必须使用 seeds 进行“签名”。
        // seeds 结构: [b"vault", signer_key, bump]
        let signer_key = ctx.accounts.signer.key();
        let seeds = &[
            b"vault",                   // 种子前缀
            signer_key.as_ref(),        // 签名者的公钥
            &[ctx.bumps.vault],         // Anchor 自动计算的 bump 值
        ];
        let signer_seeds = &[&seeds[..]]; // 需要包裹成 slice 的 slice

        // 3. 执行跨程序调用 (CPI) 进行转账。
        // 这次是从 vault 转回给 signer。
        
        // 构建 CPI 上下文，包含签名种子 (with_signer)
        let cpi_context = CpiContext::new_with_signer(
            // 目标程序：系统程序
            ctx.accounts.system_program.to_account_info(),
            // 转账参数结构体
            Transfer {
                from: ctx.accounts.vault.to_account_info(),  // 资金来源：金库 PDA
                to: ctx.accounts.signer.to_account_info(),   // 资金去向：签名者
            },
            // 签名种子，授权 PDA 进行转账
            signer_seeds,
        );

        // 获取金库当前的所有余额
        let amount = ctx.accounts.vault.lamports();

        // 执行转账
        transfer(cpi_context, amount)?;

        Ok(())
    }
}

/// 账户验证结构体：定义了 deposit 和 withdraw 指令所需的账户。
#[derive(Accounts)]
pub struct VaultAction<'info> {
    /// 签名者账户 (Signer)。
    /// 这是金库的所有者，也是唯一可以存取款的人。
    /// 需要 mut 修饰，因为转账会改变其余额。
    #[account(mut)]
    pub signer: Signer<'info>,

    /// 金库账户 (Vault)。
    /// 这是一个 PDA (Program Derived Address)，由程序控制。
    /// 
    /// 约束:
    /// - mut: 因为余额会发生变化。
    /// - seeds: 派生 PDA 的种子。这里使用字符串 "vault" 和 signer 的公钥。
    ///          这保证了每个用户都有自己唯一的金库地址。
    /// - bump: Anchor 自动寻找合法的 bump 值来生成 PDA。
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    /// 系统程序账户 (System Program)。
    /// 必须包含它，因为我们需要调用系统程序的 transfer 指令来进行 SOL 转账。
    pub system_program: Program<'info, System>,
}

/// 错误代码枚举：定义程序可能抛出的自定义错误。
#[error_code]
pub enum VaultError {
    #[msg("Vault already exists")] // 错误消息：金库已存在（不为空）
    VaultAlreadyExists,
    
    #[msg("Invalid amount")] // 错误消息：金额无效（小于免租金限额或为0）
    InvalidAmount,
}
