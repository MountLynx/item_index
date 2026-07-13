<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Index" width="80" />
</p>

<h1 align="center">Index</h1>

<p align="center">
  <strong>本地优先 · 插件扩展 · 对象管理</strong>
  <br />
  <em>类 Zotero 的个人 Wiki 管理框架 —— 基于 Tauri 2</em>
</p>

<p align="center">
  <a href="README.md">English</a>
</p>

---

## 概述

**Index** 是一个本地优先的桌面端通用对象管理框架。支持动态条目类型、多级分组、扁平标签和文件附件。通过插件和工作区配置，可以变成日历、藏书库、知识库或任何你需要的专用管理工具。

每个仓库就是磁盘上的一个普通文件夹：条目以哈希命名的子文件夹存储附件，元数据保存在本地 SQLite 数据库（`.index/index.db`）中。不上云、不锁定——数据完全属于你。

### 核心概念

| 概念 | 说明 |
|------|------|
| **条目（Item）** | 基本单位——可以是笔记、任务、书籍、联系人等。每个条目对应一个 12 位十六进制哈希文件夹。 |
| **条目类型（Type）** | 自定义字段模板。可添加文本框、勾选框等属性，灵活适配不同场景。 |
| **分组（Group）** | 多级树状目录，支持拖拽条目到不同分组。 |
| **标签（Tag）** | 扁平化的跨分组标签，一个条目可以有多个标签。 |
| **附件（Attachment）** | 拖入条目哈希文件夹的任意文件，右侧面板展示目录树。 |

### 界面预览

> *即将添加*

## 功能特性

- **动态类型系统** — 自定义条目类型，配置字段、图标和显示名
- **多级分组** — 树状组织结构，支持拖拽操作
- **扁平标签** — 跨分组筛选标签
- **文件附件** — 每个条目独立的哈希文件夹，完整目录树浏览
- **三栏布局** — 左侧（分组 + 标签）· 中间列表 · 右侧详情
- **无边框窗口** — 自定义标题栏，原生窗口控制
- **主题系统** — 亮色/暗色切换、CSS 变量自定义覆盖、预设管理
- **国际化** — 简体中文 + 英文
- **仓库看板** — 网格卡片式入口，管理多个本地仓库

## 技术栈

| 层 | 技术 |
|-----|------|
| 桌面壳 | [Tauri 2](https://v2.tauri.app/) |
| 前端 | [Vue 3.5](https://cn.vuejs.org/) + TypeScript (strict) |
| 状态管理 | [Pinia](https://pinia.vuejs.org/zh/) (组合式 API) |
| 图标 | [Tabler Icons](https://tabler.io/icons) |
| 后端 | Rust, [sqlx](https://github.com/launchbadge/sqlx) (SQLite), serde, tokio, chrono |

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) ≥ 18
- [pnpm](https://pnpm.io/zh/) ≥ 9
- [Rust](https://rustup.rs/) (stable 工具链)
- 各平台 Tauri 2 [前置依赖](https://v2.tauri.app/start/prerequisites/)

### 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器 + Tauri 窗口
pnpm dev

# 仅类型检查
pnpm build
```

开发服务器运行在 **1420 端口**。调试模式下自动打开开发者工具。

### 构建

```bash
pnpm tauri build
```

输出路径 `src-tauri/target/release/`。

## 项目结构

```
├── src/                      # Vue 3 前端
│   ├── components/           # Vue 组件 (Titlebar, Sidebar, RepoDashboard 等)
│   ├── stores/               # Pinia 状态 (repo, types, items, groups, tags, dashboard)
│   ├── locales/              # 国际化 (zh-CN.ts, en.ts)
│   ├── types/                # TypeScript 类型定义 (与 Rust 模型同步)
│   └── assets/               # theme.css, 字体
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── commands/         # IPC 命令 (repo, types, items, groups, tags, files, dashboard)
│   │   ├── models.rs         # 共享数据结构
│   │   ├── db.rs             # SQLite 连接池 + 迁移
│   │   └── safe_path.rs      # 路径穿越防护
│   ├── migrations/           # sqlx 数据库迁移
│   └── Cargo.toml
├── docs/
│   ├── design.md             # 原始构想与功能需求规格（中文）
│   └── superpowers/specs/    # 近期功能设计文档
└── CLAUDE.md                 # AI 辅助开发指南
```

### 仓库目录结构

```
~/MyIndex/                    # 一个仓库（普通文件夹）
├── .index/
│   ├── index.db              # SQLite 数据库
│   └── state.json            # 应用状态（主题等）
├── a3f2c1b8e9d4/             # 条目文件夹（12 位十六进制）
│   ├── My Note.md            # 自动生成的 Markdown 文件
│   └── image.png             # 任意附件
└── b7e1d5c3f2a8/
    └── ...
```

## 路线图

详见 [`docs/design.md`](docs/design.md)（六阶段规划）：

1. ✅ 核心引擎 — 动态类型、分组、标签、文件系统
2. ✅ 界面布局 — 三栏外壳、无边框窗口、主题系统
3. 🚧 文件管理 — 增强附件管理
4. 📋 插件系统 — 自定义视图和面板的扩展 API
5. 🤖 AI 集成 — LLM 驱动的操作与技能
6. ⚙️ 工作区配置 — 面向特定领域的预设工作区

## 参与贡献

本项目处于早期活跃开发阶段。欢迎提出反馈和想法——请提交 Issue。

## 许可证

MIT
