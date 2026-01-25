# Anchor Escrow (去中心化托管)

这是一个基于 Solana Anchor 框架实现的去中心化托管（Escrow）程序。该程序允许两方在无需信任第三方的情况下进行代币交换。

## 功能介绍

该程序包含三个核心指令：

1.  **Make (创建托管)**
    *   创建者（Maker）初始化托管账户。
    *   将代币 A 存入程序的金库（Vault）。
    *   指定希望接收的代币 B 的数量。

2.  **Take (接受托管)**
    *   接受者（Taker）接受报价。
    *   将指定数量的代币 B 转移给创建者。
    *   程序自动将金库中的代币 A 转移给接受者。
    *   交易完成，托管账户关闭。

3.  **Refund (退款)**
    *   如果在交易完成前创建者改变主意，或者无人接受报价，创建者可以取回代币 A。
    *   托管账户关闭，租金退还给创建者。

## 项目结构

代码按照模块化结构组织，位于 `programs/anchor-escrow/src/` 目录下：

*   `lib.rs`: 程序入口，定义了指令路由。
*   `state.rs`: 定义了托管账户的数据结构 (`Escrow`)。
*   `errors.rs`: 定义了自定义错误代码。
*   `instructions/`: 包含具体的指令逻辑。
    *   `make.rs`: 实现创建托管的逻辑。
    *   `take.rs`: 实现接受托管的逻辑。
    *   `refund.rs`: 实现退款逻辑。
    *   `mod.rs`: 模块导出文件。

## 环境要求

*   Anchor CLI 0.31.0 或更高版本
*   Solana CLI
*   Rust

## 构建指南

1.  **初始化依赖**
    ```bash
    yarn install
    ```

2.  **构建程序**
    ```bash
    anchor build
    ```

    *注意：如果遇到 `blake3` 依赖报错，请确保 `Cargo.toml` 中已锁定 `blake3` 版本（本项目已配置为 `=1.5.4` 以解决兼容性问题）。*

3.  **测试**
    本项目生成的 `tests/anchor-escrow.ts` 包含基础测试框架，您可以根据需要编写测试用例。
    ```bash
    anchor test
    ```

## 关键代码说明

*   **PDA (程序派生地址)**: 使用 `seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()]` 确保每个托管账户的唯一性。
*   **InitSpace**: 使用 `#[derive(InitSpace)]` 自动计算账户所需的存储空间。
*   **Token Transfer**: 使用 `anchor_spl` 提供的 CPI (跨程序调用) 功能安全地转移代币。

## 常见问题

*   **版本兼容性**: 为了支持 `discriminator` 属性，建议使用 Anchor 0.31.0+。
*   **依赖问题**: 如果构建失败，请尝试删除 `Cargo.lock` 并重新构建。

---
祝您通关顺利！


# Anchor Escrow 项目实现报告

我已按照您的要求，在 `solana-project/github/web3-solana-tasks/tasks/anchor-escrow` 目录下完成了 Anchor Escrow 项目的初始化和代码实现。

## 主要工作内容

1.  **项目初始化**
    *   使用 `anchor init anchor-escrow` 创建了项目骨架。
    *   配置了 `Cargo.toml`，添加了 `anchor-spl` 依赖，并启用了 `init-if-needed` 特性。
    *   **特别处理**: 解决了 `blake3` 依赖版本冲突问题，将其锁定为 `1.5.4`，并使用兼容性更好的 `anchor-lang 0.31.0` 版本，确保项目可以成功编译 (`anchor build` 已通过)。

2.  **代码实现 (全中文注释)**
    *   **模块化结构**: 将代码拆分为清晰的模块结构。
        *   `programs/anchor-escrow/src/lib.rs`: 程序入口，包含 Make, Take, Refund 三个指令的路由。
        *   `programs/anchor-escrow/src/state.rs`: 定义 `Escrow` 状态结构体，使用 `InitSpace` 优化存储。
        *   `programs/anchor-escrow/src/errors.rs`: 定义中文错误信息。
        *   `programs/anchor-escrow/src/instructions/`: 包含具体的业务逻辑。
            *   `make.rs`: 实现创建托管，转移代币 A 到金库。
            *   `take.rs`: 实现接受托管，交换代币并关闭账户。
            *   `refund.rs`: 实现退款逻辑。
    *   **打印信息**: 所有 `msg!()` 宏打印的内容均已修改为中文。
    *   **代码注释**: 每一行关键代码都添加了详细的中文注释，帮助您理解逻辑。

3.  **文档编写**
    *   在项目根目录创建了全中文的 [README.md](./tasks/anchor-escrow/README.md)，详细说明了项目功能、结构和构建方法，助您快速通关。

## 关键代码预览

### state.rs (状态定义)
```rust
#[account(discriminator = 1)]
pub struct Escrow {
    pub seed: u64,           // 种子
    pub maker: Pubkey,       // 创建者
    pub mint_a: Pubkey,      // 代币A
    pub mint_b: Pubkey,      // 代币B
    pub receive: u64,        // 期望接收数量
    pub bump: u8,            // PDA bump
}
```

### lib.rs (指令入口)
```rust
#[instruction(discriminator = 0)]
pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
    msg!("调用 Make 指令: seed={}, receive={}, amount={}", seed, receive, amount);
    instructions::make::make(ctx, seed, receive, amount)
}
```

现在您可以直接进入目录运行 `anchor build` 进行验证，或者查看 README 文档了解更多细节。