# TaiL CI/CD 说明

本文档介绍 TaiL 的持续集成和部署配置。

## 概述

TaiL 使用 GitHub Actions 进行 CI/CD，确保代码质量和构建一致性。

## CI 工作流

### 工作流文件

`.github/workflows/ci.yml`

### 检查内容

每次提交都会执行：

1. **代码格式检查**
   ```yaml
   - run: nix fmt --check
   ```

2. **Flake 检查**
   ```yaml
   - run: nix flake check
   ```

3. **单元测试**
   ```yaml
   - run: nix develop --command cargo test --lib --workspace
   ```

4. **集成测试**
   ```yaml
   - run: nix develop --command cargo test --test '*' --workspace
   ```

5. **Clippy 检查**
   ```yaml
   - run: nix develop --command cargo clippy --workspace --all-targets -- -D warnings
   ```

6. **Release 构建**
   ```yaml
   - run: nix build .#tail-app .#tail-service
   ```

### 触发条件

CI 在以下情况触发：

- Push 到 `main` 分支
- Push 到 `develop` 分支
- 创建 Pull Request
- 手动触发

## 本地验证

提交前可以在本地运行相同的检查：

```bash
# 格式检查
nix fmt --check

# 或自动格式化
nix fmt

# Flake 检查
nix flake check

# 运行测试
nix develop --command cargo test --workspace

# Clippy 检查
nix develop --command cargo clippy --workspace --all-targets

# Release 构建
nix build .#tail-app .#tail-service
```

或使用 just：

```bash
just check    # 运行所有检查
just test     # 运行测试
just clippy   # 运行 Clippy
just fmt      # 格式化代码
```

## 状态徽章

```markdown
![CI](https://github.com/vitus213/tail/workflows/CI/badge.svg)
```

## 发布流程

### 版本号

遵循语义化版本 [Semantic Versioning](https://semver.org/)：

- `MAJOR.MINOR.PATCH`
  - MAJOR：不兼容的 API 变更
  - MINOR：向后兼容的功能新增
  - PATCH：向后兼容的 Bug 修复

### 创建 Release

1. 更新版本号（如有必要）

2. 更新 `CHANGELOG.md`

3. 创建 Git tag：

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

4. GitHub Actions 会自动：
   - 构建 Release 二进制文件
   - 创建 GitHub Release
   - 上传构建产物

## Cachix 集成

项目使用 Cachix 缓存 Nix 构建结果：

```yaml
- uses: cachix/cachix-action@v12
  with:
    name: tail
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
```

这可以显著加快 CI 和用户构建速度。

## 故障排查

### CI 失败

如果 CI 失败：

1. 查看失败步骤的日志
2. 在本地复现问题
3. 修复并推送新提交

### 超时

如果构建超时：

```yaml
- run: nix build .#tail-app
  timeout-minutes: 30
```

### 缓存问题

如果缓存导致问题：

```bash
# 清理本地缓存
nix-store --delete $(nix-store -q --requisites /nix/var/nix/profiles/default)
```
