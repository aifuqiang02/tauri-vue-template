# Tauri + Vue 桌面应用模板

## 特性

- Vue 3 + TypeScript
- Tailwind 4
- Vite + AutoImport
- Vitest 单元测试
- GitHub Actions CI/CD

## 准备环境

- 导出 `github.com` 证书并导入“受信任的发布者”是必须先完成的环境准备，否则依赖下载、NSIS 下载或应用内更新访问 GitHub 时可能失败，具体操作见 [常见问题](./常见问题.md)

## 快速开始

1. 安装依赖：

```sh
pnpm install
```

2. 启动开发：

```sh
pnpm tauri dev
```

## 命令

- `pnpm tauri dev` - 启动开发
- `pnpm tauri build` - 构建生产版本
- `pnpm test` - 运行测试

## 应用内更新

模板已接入 Tauri v2 updater，设置页里可以直接检查更新并安装。

本地开发或普通构建不会默认生成 updater 产物；只有 GitHub Release 工作流会额外加载 `src-tauri/tauri.updater.conf.json` 来生成 `latest.json` 和签名文件。

发布前需要准备这些 GitHub 配置：

- `secrets.TAURI_SIGNING_PRIVATE_KEY`
- `secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `vars.TAURI_UPDATER_PUBLIC_KEY`

可以先用 Tauri CLI 生成签名密钥：

```sh
pnpm tauri signer generate
```

发布工作流会自动把更新地址设置成当前仓库的 GitHub Releases 最新下载地址：

```text
https://github.com/<owner>/<repo>/releases/latest/download/latest.json
```

## 项目结构

- `src/` - 前端代码 (Vue)
- `src-tauri/` - 后端代码 (Rust)
