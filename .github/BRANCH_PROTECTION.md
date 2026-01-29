# 分支保护规则配置指南

## 概述

为了确保代码质量和项目稳定性，需要在GitHub仓库中配置分支保护规则。这些规则将确保所有Pull Request必须通过CI检查才能合并到主分支。

## 配置步骤

### 1. 访问分支保护设置

1. 进入GitHub仓库页面
2. 点击 **Settings** 标签
3. 在左侧菜单中选择 **Branches**
4. 点击 **Add rule** 或编辑现有规则

### 2. 配置主分支保护 (main)

**Branch name pattern**: `main`

**保护规则设置**:

- ✅ **Require a pull request before merging**
  - ✅ Require approvals: `1` (至少需要1个审批)
  - ✅ Dismiss stale PR approvals when new commits are pushed
  - ✅ Require review from code owners (如果有CODEOWNERS文件)

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - **Required status checks** (添加以下检查):
    - `Test Suite (stable)`
    - `Test Suite (beta)` 
    - `Test Suite (nightly)`
    - `Build Check (ubuntu-latest, stable)`
    - `Build Check (windows-latest, stable)`
    - `Build Check (macos-latest, stable)`
    - `Security Audit`
    - `Code Coverage`
    - `Minimum Rust Version Check`
    - `PR Validation`

- ✅ **Require conversation resolution before merging**

- ✅ **Require signed commits** (推荐)

- ✅ **Require linear history** (推荐，保持提交历史整洁)

- ✅ **Include administrators** (管理员也需要遵循规则)

### 3. 配置开发分支保护 (develop)

如果使用develop分支，可以配置相似但稍微宽松的规则：

**Branch name pattern**: `develop`

- ✅ **Require a pull request before merging**
  - Require approvals: `1`
- ✅ **Require status checks to pass before merging**
  - 至少包含: `Test Suite (stable)`, `PR Validation`

## CI工作流说明

### 主要工作流 (.github/workflows/ci.yml)

- **测试矩阵**: 在stable、beta、nightly版本的Rust上运行测试
- **多平台构建**: 在Ubuntu、Windows、macOS上验证构建
- **代码质量**: 运行rustfmt、clippy检查
- **安全审计**: 使用cargo-audit检查依赖安全性
- **代码覆盖率**: 生成并上传到Codecov
- **性能检查**: 基础性能验证

### PR检查工作流 (.github/workflows/pr-checks.yml)

- **PR标题验证**: 确保遵循conventional commits格式
- **API变更检测**: 检查是否有破坏性变更
- **综合检查**: 格式化、静态分析、测试

## 推荐的PR流程

1. **创建功能分支**: `git checkout -b feat/your-feature`
2. **开发和测试**: 确保本地测试通过
3. **提交代码**: 使用conventional commits格式
4. **创建PR**: 填写详细的PR描述
5. **等待CI**: 所有自动检查必须通过
6. **代码审查**: 获得至少1个审批
7. **合并**: 使用"Squash and merge"保持历史整洁

## 本地开发建议

在提交PR前，建议在本地运行以下检查：

```bash
# 格式化检查
cargo fmt --all -- --check

# 静态分析
cargo clippy --all-targets --all-features -- -D warnings

# 运行测试
cargo test --all-features

# 构建检查
cargo build --release

# 安全审计
cargo audit
```

## 故障排除

### 常见CI失败原因

1. **格式化失败**: 运行 `cargo fmt --all`
2. **Clippy警告**: 修复代码中的警告
3. **测试失败**: 确保所有测试在本地通过
4. **构建失败**: 检查跨平台兼容性

### 绕过保护规则

在紧急情况下，仓库管理员可以：
1. 临时禁用分支保护
2. 使用管理员权限强制合并
3. 合并后立即重新启用保护规则

**注意**: 这应该只在紧急修复时使用，并需要在团队中说明原因。

## 维护

定期检查和更新：
- CI工作流配置
- 分支保护规则
- 依赖项安全性
- 性能基准测试结果

这些配置确保了代码质量和项目的长期可维护性。