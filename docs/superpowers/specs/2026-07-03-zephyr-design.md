# Zephyr 高速 Markdown 阅读器 — 设计规格

## 概述

Zephyr 是一个使用 Tauri 2 + Rust + Vanilla JS 构建的高速 Markdown 阅读器，渲染效果与 GitHub 上的 Markdown 预览风格一致。

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 桌面框架 | Tauri 2 | 跨平台桌面应用容器 |
| 后端语言 | Rust | 核心渲染、文件 I/O、搜索 |
| 前端 | Vanilla JS + HTML + CSS | 界面交互与展示 |
| Markdown 解析 | `comrak` (Rust) | Markdown→HTML 转换 |
| 语法高亮 | `syntect` (Rust) | 代码块语法高亮 |
| 数学公式 | `KaTeX` (JS) | LaTeX 数学公式渲染 |
| 图表 | `Mermaid` (JS) | 流程图/时序图/甘特图等 |
| 文件监听 | `notify` (Rust) | 外部文件修改检测 |

## 核心需求

| # | 需求 | 说明 |
|---|------|------|
| 1 | 多种打开方式 | 应用内文件浏览器、拖拽文件、命令行参数 |
| 2 | 完整 GFM 支持 | 基本语法 + GFM 扩展 + 语法高亮/LaTeX/Mermaid/脚注 |
| 3 | 可切换侧边栏 | 默认隐藏，可唤出文件树面板 |
| 4 | 高速渲染 | Rust 端解析+高亮，大文件毫秒级处理 |
| 5 | 主题支持 | 亮色（默认）/ 深色，CSS 变量切换 |
| 6 | 文档内导航 | 目录/大纲视图 + 文档内搜索 |
| 7 | 文件监听 | 外部修改自动刷新内容 |

## 架构

### Rust 后端

```
src-tauri/src/
├── main.rs              # 程序入口
├── lib.rs               # Tauri Builder 配置
├── commands/            # Tauri 命令
│   ├── mod.rs
│   ├── file.rs          # 打开文件、读取目录、拖拽处理
│   ├── parse.rs         # 调用渲染管道返回 HTML
│   └── search.rs        # 文档内搜索
├── parser/              # Markdown 渲染管道
│   ├── mod.rs
│   ├── markdown.rs      # comrak 解析 Markdown→HTML
│   └── highlighter.rs   # syntect 代码语法高亮
└── watcher.rs           # notify 文件监听
```

### 渲染管道

```
原始 Markdown 文本
    ↓ (comrak::parse_document)
AST
    ↓ (自定义遍历：提取标题结构用于大纲)
    ↓ (syntect 对代码块逐个高亮)
    ↓ (comrak 格式化输出 HTML)
完整 HTML → Tauri invoke → 前端
```

### 前端

```
src/
├── index.html              # 主页面结构
├── styles.css              # 全局样式
├── github-markdown.css     # GitHub 风格 Markdown CSS
├── main.js                 # 入口 — 初始化、事件路由
├── state.js                # 应用状态
└── components/
    ├── sidebar.js          # 文件树面板（可切换）
    ├── viewer.js           # HTML 内容渲染 + KaTeX/Mermaid 后处理
    ├── outline.js          # 文档大纲导航
    ├── search.js           # 文档内搜索
    └── theme.js            # 亮色/深色主题切换
```

### 布局

```
┌───────────────────────────────┐
│  ┌──────────┐ ┌─────────────┐ │
│  │ 侧边栏    │ │ 工具栏      │ │
│  │ (可收起)  │ │ ☰ ⚙ 🔍 🌙 │ │
│  │          │ ├─────────────┤ │
│  │ 文件树    │ │             │ │
│  │ ─────     │ │ Markdown   │ │
│  │ 目录大纲   │ │ 内容区域    │ │
│  │          │ │             │ │
│  └──────────┘ └─────────────┘ │
└───────────────────────────────┘
```

### IPC 通信

- **Tauri Commands（JS invoke Rust）：**
  - `open_file(path) → MarkdownResult { html, headings, title, word_count }`
  - `open_directory(path) → Vec<FileEntry>`
  - `search_in_document(query) → Vec<SearchMatch>`
  - `get_file_tree(path) → FileTree`

- **Tauri Events（Rust → JS）：**
  - `file-changed` — 通知外部文件已修改

## 数据模型

```rust
#[derive(Serialize)]
struct MarkdownResult {
    html: String,
    title: String,
    headings: Vec<Heading>,
    word_count: u32,
}

#[derive(Serialize)]
struct Heading {
    level: u8,
    text: String,
    id: String,       // GitHub 风格 anchor id
    children: Vec<Heading>,
}

#[derive(Serialize)]
struct SearchMatch {
    line: u32,
    column: u32,
    context: String,
}

#[derive(Serialize)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    is_markdown: bool,
}
```

## 错误处理

| 场景 | 处理 |
|------|------|
| 文件不存在 | Rust 返回 Err，前端显示友好提示 |
| 非 Markdown 文件 | 根据扩展名判断并提示 |
| 超大文件（>10MB） | 异步流式读取，显示进度 |
| KaTeX 错误 | 回退显示原始 LaTeX 文本 |
| Mermaid 错误 | 回退显示原始代码块 |
| 外部文件被删除 | 显示提示"文件已被删除" |
| 空文件 | 显示"空文件"提示 |

## 清单

- [x] 需求明确
- [x] 架构确定（方案 A：Rust 主导解析）
- [x] 前端设计确定
- [x] IPC 通信确定
- [x] 错误处理确定
