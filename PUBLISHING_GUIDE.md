# shine-rs 发布到 crates.io 指南

本指南将帮助你将 shine-rs 项目发布到 crates.io。

## 发布前准备

### 1. 账户准备

首先，你需要在 [crates.io](https://crates.io) 上创建账户：

1. 访问 https://crates.io
2. 使用 GitHub 账户登录
3. 生成 API Token：
   - 点击右上角用户名 → Account Settings
   - 在 "API Tokens" 部分点击 "New Token"
   - 输入 token 名称（如 "shine-rs-publish"）
   - 复制生成的 token

### 2. 本地配置

在本地配置 cargo 登录：

```bash
cargo login <your-api-token>
```

### 3. 项目状态检查

运行我们的准备脚本：

```powershell
# Windows PowerShell
.\scripts\prepare_release.ps1

# 或者手动执行以下步骤：
```

```bash
# 1. 检查编译
cargo check

# 2. 运行基础测试
cargo test --lib

# 3. 检查包内容
cargo package --list --allow-dirty

# 4. 干运行发布
cargo publish --dry-run --allow-dirty --registry crates-io
```

## 发布步骤

### 第一次发布 (v0.1.0)

```bash
# 确保所有更改都已提交到 git
git add .
git commit -m "Prepare for v0.1.0 release"
git tag v0.1.0
git push origin main --tags

# 发布到 crates.io
cargo publish --registry crates-io
```

### 后续版本发布

1. **更新版本号**：编辑 `Cargo.toml` 中的 `version` 字段
2. **更新 CHANGELOG**：记录新功能和修复
3. **提交更改**：
   ```bash
   git add .
   git commit -m "Bump version to v0.x.x"
   git tag v0.x.x
   git push origin main --tags
   ```
4. **发布**：
   ```bash
   cargo publish --registry crates-io
   ```

## 发布后验证

### 1. 检查 crates.io

- 访问 https://crates.io/crates/shine-rs
- 确认包信息正确显示
- 检查文档链接是否工作

### 2. 测试安装

在另一个项目中测试安装：

```bash
# 创建测试项目
cargo new test-shine-rs
cd test-shine-rs

# 添加依赖
cargo add shine-rs

# 测试基本功能
```

创建 `src/main.rs`：

```rust
use shine_rs::mp3_encoder::{Mp3EncoderConfig, encode_pcm_to_mp3, StereoMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 生成测试音频数据
    let pcm_data: Vec<i16> = (0..44100).map(|i| (i as f64 * 0.1).sin() as i16 * 1000).collect();
    
    // 配置编码器
    let config = Mp3EncoderConfig::new()
        .sample_rate(44100)
        .bitrate(128)
        .channels(1)
        .stereo_mode(StereoMode::Mono);
    
    // 编码
    let mp3_data = encode_pcm_to_mp3(config, &pcm_data)?;
    
    println!("Successfully encoded {} bytes of MP3 data", mp3_data.len());
    Ok(())
}
```

```bash
cargo run
```

### 3. 文档检查

- 访问 https://docs.rs/shine-rs
- 确认文档正确生成
- 检查示例代码是否可运行

## 包配置说明

### Cargo.toml 关键配置

```toml
[package]
name = "shine-rs"
version = "0.1.0"
edition = "2021"
authors = ["wshon <wshon@example.com>"]
description = "A pure Rust MP3 encoder based on the shine library, providing complete MPEG Layer III encoding functionality"
license = "LGPL-2.1-or-later"
repository = "https://github.com/wshon/shine-rs"
homepage = "https://github.com/wshon/shine-rs"
documentation = "https://docs.rs/shine-rs"
readme = "README.md"
keywords = ["mp3", "audio", "encoder", "codec", "shine"]
categories = ["multimedia::audio", "encoding"]
exclude = [
    "ref/*",
    "testing/*", 
    "tools/*",
    "scripts/*",
    "docs/*",
    "*.mp3",
    "*.wav",
    "*.pdb",
    ".git*",
    ".claude/*"
]
```

### 包含的文件

发布包将包含以下文件：
- 所有 `src/` 目录下的源代码
- `examples/` 目录下的示例
- `README.md`、`LICENSE`、`Cargo.toml`
- 基础测试文件

### 排除的文件

以下文件不会包含在发布包中：
- `ref/` - shine 参考实现
- `testing/` - 测试数据和集成测试
- `tools/` - 命令行工具
- `scripts/` - 构建脚本
- `docs/` - 项目文档
- 生成的音频文件 (*.mp3, *.wav)

## 常见问题

### Q: 发布失败，提示 "crates-io is replaced with non-remote-registry"

A: 这通常是因为本地有镜像配置。使用 `--registry crates-io` 参数：
```bash
cargo publish --registry crates-io
```

### Q: 包大小过大

A: 检查 `exclude` 配置，确保排除了不必要的文件：
```bash
cargo package --list --allow-dirty
```

### Q: 文档生成失败

A: 确保所有公共 API 都有文档注释，并且没有编译错误：
```bash
cargo doc --no-deps
```

### Q: 依赖版本冲突

A: 使用兼容的版本范围，避免过于严格的版本限制：
```toml
[dependencies]
thiserror = "1.0"  # 好
thiserror = "=1.0.69"  # 避免
```

## 版本管理策略

### 语义化版本

遵循 [Semantic Versioning](https://semver.org/)：

- **MAJOR** (1.0.0): 不兼容的 API 更改
- **MINOR** (0.1.0): 向后兼容的功能添加
- **PATCH** (0.1.1): 向后兼容的错误修复

### 发布节奏

建议的发布策略：
- **0.1.x**: 初始版本，基础功能
- **0.2.x**: 添加高级功能，API 稳定化
- **1.0.0**: 稳定版本，API 承诺向后兼容

## 维护指南

### 定期更新

- 定期更新依赖项
- 修复安全漏洞
- 改进文档和示例

### 社区支持

- 及时回应 GitHub Issues
- 审查和合并 Pull Requests
- 维护 CHANGELOG

### 监控

- 关注下载统计
- 收集用户反馈
- 监控构建状态

## 成功发布检查清单

- [ ] 代码编译无错误无警告
- [ ] 基础测试通过
- [ ] 文档完整且正确
- [ ] README.md 信息准确
- [ ] LICENSE 文件存在
- [ ] 版本号正确
- [ ] Git 标签已创建
- [ ] 干运行成功
- [ ] 实际发布成功
- [ ] crates.io 页面正确显示
- [ ] docs.rs 文档生成成功
- [ ] 安装测试通过

恭喜！你的 shine-rs 包现在已经可以供全世界的 Rust 开发者使用了！🎉