import fs from 'fs';
import path from 'path';
import os from 'os';
import bs58 from 'bs58';

// 定义 Solana 默认密钥对文件的路径，通常在用户主目录下的 .config/solana/id.json
const KEYPAIR_PATH = path.join(os.homedir(), '.config', 'solana', 'id.json');

// 定义 .env 文件的路径，在当前脚本运行的目录下
const ENV_PATH = path.join(process.cwd(), '.env');

async function main() {
    try {
        // 1. 检查 Solana 密钥文件是否存在
        if (!fs.existsSync(KEYPAIR_PATH)) {
            console.error(`❌ 未找到 Solana 密钥文件: ${KEYPAIR_PATH}`);
            console.log('💡 请先安装 Solana CLI 并运行 `solana-keygen new` 生成密钥。');
            process.exit(1);
        }

        // 2. 读取密钥文件内容
        console.log(`📖 正在读取密钥文件: ${KEYPAIR_PATH}`);
        const keypairContent = fs.readFileSync(KEYPAIR_PATH, 'utf-8');
        
        // 3. 将 JSON 内容解析为数组
        const keypairArray = JSON.parse(keypairContent);
        
        // 4. 将数字数组转换为 Uint8Array
        const secretKey = new Uint8Array(keypairArray);
        
        // 5. 使用 bs58 将私钥编码为字符串
        const secretBase58 = bs58.encode(secretKey);
        console.log('✅ 成功获取并编码私钥。');

        // 6. 读取现有的 .env 文件内容（如果存在）
        let envContent = '';
        if (fs.existsSync(ENV_PATH)) {
            envContent = fs.readFileSync(ENV_PATH, 'utf-8');
        }

        // 7. 更新或添加 SECRET 变量
        // 使用正则表达式查找现有的 SECRET=... 行
        const secretRegex = /^SECRET=.*$/m;
        
        if (secretRegex.test(envContent)) {
            // 如果存在，则替换它
            console.log('🔄 正在更新 .env 文件中的 SECRET...');
            envContent = envContent.replace(secretRegex, `SECRET="${secretBase58}"`);
        } else {
            // 如果不存在，则追加到文件末尾
            console.log('➕ 正在向 .env 文件添加 SECRET...');
            // 确保文件末尾有换行符
            if (envContent && !envContent.endsWith('\n')) {
                envContent += '\n';
            }
            envContent += `SECRET="${secretBase58}"\n`;
        }

        // 8. 将更新后的内容写回 .env 文件
        fs.writeFileSync(ENV_PATH, envContent);
        console.log(`🎉 成功将私钥写入到: ${ENV_PATH}`);
        
    } catch (error) {
        console.error('❌ 发生错误:', error);
    }
}

main();
