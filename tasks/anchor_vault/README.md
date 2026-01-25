# Anchor Vault (金库) 作业指南

这个项目是一个简单的 Solana Anchor 程序，实现了一个基本的金库（Vault）功能。用户可以将 SOL 存入金库，并在之后取回。

该项目是根据 Blueshift 挑战 "Anchor Vault" 完成的，包含了完整的中文注释，帮助你快速理解和通关。

## 📁 目录结构

- `programs/anchor_vault/src/lib.rs`: **核心代码文件**。包含了存款（Deposit）和取款（Withdraw）的逻辑。代码中已添加详细的逐行中文注释。
- `Anchor.toml`: Anchor 的配置文件，已配置好作业要求的 Program ID。

## 🎯 核心功能

这个程序主要演示了以下 Anchor 和 Solana 的核心概念：

1.  **PDA (Program Derived Address)**:
    - 金库账户是一个 PDA，它没有私钥，而是通过种子（Seeds）和程序 ID 派生出来的。
    - 种子设置为 `[b"vault", signer.key().as_ref()]`，这意味着每个用户都有一个属于自己的唯一金库地址。
    
2.  **CPI (Cross Program Invocation)**:
    - 程序本身不直接处理转账，而是通过 CPI 调用 Solana 的 **System Program** 来执行 SOL 的转移。
    - `deposit`: 从用户账户转账到金库 PDA。
    - `withdraw`: 从金库 PDA 转账回用户账户（需要 PDA 签名）。

## 🚀 快速开始

### 1. 安装依赖
确保你已经安装了 Rust, Solana CLI 和 Anchor。

### 2. 构建项目
在项目根目录下运行：
```bash
anchor build
```
这会编译 Rust 代码并在 `target/deploy` 目录下生成程序二进制文件。

### 3. 测试 (可选)
运行测试脚本（如果有的话）：
```bash
anchor test
```

## 📝 代码解析 (通关秘籍)

### Program ID
作业要求必须使用特定的 Program ID：
`22222222222222222222222222222222222222222222`
这已经在 `lib.rs` 和 `Anchor.toml` 中配置好了，**请勿修改**，否则可能无法通过作业验证。

### 账户结构 (VaultAction)
我们定义了一个统一的上下文 `VaultAction` 用于存款和取款，它包含：
- `signer`: 操作者（用户），必须签名。
- `vault`: 金库账户（PDA），使用 `seeds` 和 `bump` 约束。
- `system_program`: 系统程序，用于转账。

### 错误处理
- `VaultAlreadyExists`: 存款时检查金库是否为空，防止覆盖。
- `InvalidAmount`: 存款金额必须大于免租金最低额；取款时金库必须有余额。

## 💡 常见问题

**Q: 为什么存款时要检查 `lamports == 0`?**
A: 为了简单起见，这个作业假设每次存款都是到一个新的或空的金库。如果金库里已经有钱，直接覆盖可能会导致逻辑混乱（虽然技术上是可以追加的，但作业要求检查为空）。

**Q: 为什么取款时不需要签名者签名?**
A: 取款时，`signer` (用户) 仍然是签名者，但更重要的是，资金是从 `vault` (PDA) 转出的。PDA 不能像普通账户那样签名，它需要程序通过 `seeds` 和 `bump` 来“证明”它是该 PDA 的所有者（CpiContext::new_with_signer）。

祝你通关顺利！
