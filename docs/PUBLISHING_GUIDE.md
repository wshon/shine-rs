# shine-rs 发布到 crates.io 指南

本指南记录了 shine-rs 项目的实际发布流程，基于 v0.1.5 发布的真实经验整理。

## 项目结构

shine-rs 是一个 Cargo workspace，包含两个包：

- `crate/` — `shine-rs` 库（发布的 crate）
- 根目录 — `shine-rs-cli` 二进制（内部工具，不发布）

**只发布库 `shine-rs`，不发布 CLI。**

## 发布前准备

### 1. 账户准备

在 [crates.io](https://crates.io) 创建账户并生成 API Token：

1. 访问 https://crates.io 用 GitHub 登录
2. Account Settings → API Tokens → New Token
3. 复制 token

本地配置 cargo 登录（只需一次）：

```bash
cargo login <your-api-token>
```

### 2. 状态检查

发布前确保：

- 所有测试通过
- 无编译警告
- 所有更改已提交

```bash
# 全量测试
cargo test

# 编译检查
cargo check
```

## 发布步骤（逐项执行，每步确认后再继续）

### 第 1 步：更新版本号

编辑 `crate/Cargo.toml`，修改 `version` 字段：

```toml
[package]
name = "shine-rs"
version = "0.1.5"  # ← 递增
```

### 第 2 步：更新 CHANGELOG

编辑 `crate/CHANGELOG.md`，添加新版本条目。格式参考已有内容。

### 第 3 步：同步根 Cargo.toml 的依赖版本

**重要**：根目录 `Cargo.toml` 中 `shine-rs` 依赖必须指定 `version`，否则 `cargo publish` 会报错。

编辑根 `Cargo.toml`：

```toml
[dependencies]
shine-rs = { path = "crate", version = "0.1.5" }  # ← 加 version
```

### 第 4 步：编译验证

```bash
cargo check
```

### 第 5 步：干运行

```bash
cargo publish -p shine-rs --dry-run
```

检查输出无报错后再继续。

### 第 6 步：提交并推送

```bash
git add crate/Cargo.toml crate/CHANGELOG.md Cargo.toml
git commit -m "chore(release): bump version to 0.1.5"
git tag -a v0.1.5 -m "v0.1.5 — 简短描述"
git push origin main --tags
```

**禁止 force push 到 tags。**

### 第 7 步：发布

```bash
cargo publish -p shine-rs
```

只发库，不发 CLI。

### 第 8 步：验证

- https://crates.io/crates/shine-rs — 确认版本和描述正确
- https://docs.rs/shine-rs — 确认文档生成
- `cargo search shine-rs` — 确认索引可见

## 常见问题

### Q: 报错 "all dependencies must have a version requirement"

根 `Cargo.toml` 中 `shine-rs` 依赖缺少 `version` 字段。添加即可。

### Q: 报错 "failed to verify manifest"

先运行 `cargo publish -p shine-rs --dry-run` 定位问题。

### Q: tag 推错了怎么办

不要 force push。如果需要修正：
1. 本地删除并重建 tag：`git tag -d v0.1.5 && git tag -a v0.1.5 -m "..."`
2. 如果已推送：`git push origin :refs/tags/v0.1.5`（删除远程 tag）
3. 再重新推送

## 检查清单

发布前逐项核对：

- [ ] `cargo test` 全部通过
- [ ] `crate/Cargo.toml` 版本号已递增
- [ ] `crate/CHANGELOG.md` 已更新
- [ ] 根 `Cargo.toml` 中 `shine-rs` 依赖包含 `version`
- [ ] `cargo publish -p shine-rs --dry-run` 无报错
- [ ] commit 消息符合 `chore(release): bump version to vX.Y.Z`
- [ ] tag 已创建并推送（非 force）
- [ ] `cargo publish -p shine-rs` 成功
- [ ] crates.io 页面和 docs.rs 正确显示
