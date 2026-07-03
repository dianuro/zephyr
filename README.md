# Zephyr — 高速 Markdown 阅读器

<p align="center">
  <img src="https://img.shields.io/badge/built_with-Tauri_2-ffc131?style=flat-square" alt="Tauri 2"/>
  <img src="https://img.shields.io/badge/backend-Rust-dea584?style=flat-square" alt="Rust"/>
  <img src="https://img.shields.io/badge/frontend-Vanilla_JS-f0db4f?style=flat-square" alt="Vanilla JS"/>
</p>

Zephyr 是一个用 **Tauri 2 + Rust + Vanilla JS** 构建的高速 Markdown 阅读器。渲染效果接近 GitHub 阅读体验，强调秒开大文件、完整 GFM 支持和简洁界面。

## ✨ 特性

- **⚡ 毫秒级渲染** — Rust 端 `comrak` 解析 Markdown + `syntect` 语法高亮，大文件秒开
- **📄 完整 GFM 支持**
  - 标题、粗斜体、列表、表格、任务列表、删除线、自动链接
  - 代码语法高亮（亮/暗双主题）
  - LaTeX 数学公式（KaTeX）
  - Mermaid 图表
  - 脚注
- **📂 文件浏览**
  - VS Code 风格可折叠文件树（懒加载）
  - 大纲面板（标题导航）
  - 原生文件对话框
  - 拖拽打开文件
  - CLI 参数直接打开：`zephyr ./README.md`
- **🎨 双主题** — 亮色/暗色，一键切换
  - 使用 GitHub 官方 `github-markdown-css` 渲染风格
  - 所有颜色通过 TOML 配置文件自定义
- **🔍 文档内搜索** — 高亮所有匹配项，Enter 快速跳转
- **🔄 自动刷新** — 外部修改文件时自动重新加载
- **📋 代码块复制** — 悬停出现复制按钮，一键复制
- **🔗 外部链接** — 点击自动用系统默认浏览器打开
- **🪟 自定义标题栏** — 原生窗口控件（最小化、最大化、关闭）

## 🚀 快速开始

### 前置要求

- [Rust](https://www.rust-lang.org/)（≥ 1.78）
- [Node.js](https://nodejs.org/)（用于 Tauri CLI）
- 系统依赖：请参考 [Tauri 2 系统依赖文档](https://v2.tauri.app/start/prerequisites/)

### 启动

```bash
# 安装 Tauri CLI
npm install -g @tauri-apps/cli

# 进入项目目录
cd zephyr

# 开发模式（支持热重载）
cargo tauri dev

# 打开指定文件
cargo tauri dev -- ./README.md
```

### 构建

```bash
cargo tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 📖 使用

### 打开文件

| 方式 | 操作 |
|------|------|
| 命令行 | `zephyr path/to/file.md` |
| 文件对话框 | 点击工具栏文件夹图标 或 `Ctrl+O` |
| 拖拽 | 将 `.md` 文件拖入窗口 |
| 文件树 | 展开目录，点击文件 |

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl+\` | 切换侧边栏 |
| `Ctrl+F` | 搜索 |
| `Ctrl+O` | 打开文件 |
| `F11` | 全屏 |

### 侧边栏

- **文件树** — 以当前文件所在目录为根，懒加载展开/折叠
- **大纲** — 自动提取 Markdown 标题结构，点击跳转

### 主题

点击工具栏太阳/月亮图标切换亮色/暗色。主题配置持久化到 `localStorage`。

## ⚙️ 配置

主题颜色通过配置文件定义，位于：

```
~/.config/zephyr/theme/default/
├── light.toml
└── dark.toml
```

首次启动时自动生成。修改后重启应用生效。

```toml
# ~/.config/zephyr/theme/default/dark.toml
[app]
background = "#0d1117"
sidebar_bg = "#161b22"
toolbar_bg = "#161b22"
content_bg = "#0d1117"
hover_bg = "#1c2128"

[text]
primary = "#e6edf3"
secondary = "#848d97"
muted = "#6e7681"

[border]
default = "#30363d"
muted = "#21262d"

[accent]
default = "#2f81f7"
hover = "#58a6ff"

[scrollbar]
thumb = "#30363d"
thumb_hover = "#6e7681"

[search]
highlight_bg = "rgba(187,128,9,0.15)"
active_bg = "rgba(187,128,9,0.4)"

[syntax]
theme = "base16-ocean.dark"
```

### 可用的语法高亮主题

| 主题 | 模式 |
|------|------|
| `InspiredGitHub` | 亮色 |
| `Solarized (light)` | 亮色 |
| `base16-ocean.light` | 亮色 |
| `base16-ocean.dark` | 暗色 |
| `base16-eighties.dark` | 暗色 |
| `base16-mocha.dark` | 暗色 |
| `Solarized (dark)` | 暗色 |

在配置文件的 `[syntax] theme` 中设置你喜欢的主题。

## 🏗 架构

```
zephyr/
├── src/                          # 前端（Vanilla JS）
│   ├── index.html                # 入口
│   ├── main.js                   # 初始化、窗口控制、拖拽、快捷键
│   ├── state.js                  # 共享状态
│   ├── styles.css                # UI 样式（Primer 色值）
│   └── components/
│       ├── viewer.js             # Markdown 渲染、文件打开、重新渲染、复制按钮、外部链接
│       ├── sidebar.js            # 文件树（懒加载）、大纲面板切换
│       ├── outline.js            # 标题导航
│       ├── search.js             # 文档内搜索
│       └── theme.js              # 亮/暗主题切换、TOML 配置加载
├── src-tauri/
│   └── src/
│       ├── lib.rs                # Tauri 入口、命令注册、插件初始化
│       ├── main.rs               # 桌面入口
│       ├── commands/
│       │   ├── file.rs           # 文件操作（打开、目录浏览、文件树、选择对话框）
│       │   ├── parse.rs          # Markdown 渲染命令
│       │   └── search.rs         # 文档搜索
│       ├── parser/
│       │   └── markdown.rs       # comrak 解析 + syntect 高亮 + heading 提取
│       ├── watcher.rs            # 文件变更监听（notify）
│       └── theme_config.rs       # TOML 主题配置文件管理
└── docs/superpowers/
    ├── specs/                    # 设计规格
    └── plans/                    # 实现计划
```

### 数据流

```
用户打开文件
  → CLI 参数 / 对话框 / 拖拽
  → viewer.js: openFile(path)
  → Rust: open_file (读文件 + comrak 渲染 HTML + syntect 高亮)
  → 返回 { html, metadata, raw_content }
  → 前端注入 #content
  → 后处理：KaTeX 公式、Mermaid 图表、复制按钮
  → 更新侧边栏：文件树（首次建树，后续只更新高亮）、大纲
  → 启动文件变更监听
```

## 🧪 测试

```bash
cd src-tauri
cargo test
```

当前 20 个 Rust 测试，覆盖 Markdown 渲染、标题提取、搜索、文件操作、路径解析等。

## 🛠 技术栈

| 层 | 技术 |
|----|------|
| **框架** | Tauri 2 |
| **后端** | Rust + comrak（Markdown 解析）+ syntect（语法高亮）+ notify（文件监听） |
| **前端** | Vanilla JS (ES Modules) |
| **样式** | github-markdown-css v5.5.1（Primer 色值）+ KaTeX + Mermaid |
| **配置** | TOML（~/.config/zephyr/） |
| **构建** | cargo tauri build |

## 📝 许可

MIT
