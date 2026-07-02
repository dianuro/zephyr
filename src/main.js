import state from './state.js';
import { initSidebar, toggleSidebar } from './components/sidebar.js';
import { openFile } from './components/viewer.js';
import { initSearch } from './components/search.js';
import { initTheme } from './components/theme.js';

async function init() {
  // 初始化各组件
  initSidebar();
  initSearch();
  initTheme();

  // 拖拽支持
  setupDragDrop();

  // 文件变化监听
  setupFileWatcher();

  // CLI 参数支持
  handleCliArgs();

  // 显示侧边栏快捷键
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
      e.preventDefault();
      toggleSidebar();
    }
  });

  console.log('Zephyr 已启动');
}

function setupDragDrop() {
  document.addEventListener('dragover', (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  });

  document.addEventListener('drop', async (e) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer.files);
    const mdFile = files.find(f => f.name.endsWith('.md') || f.name.endsWith('.markdown'));

    if (mdFile) {
      // 在 Tauri 中，拖拽的文件可以通过 path 属性获取路径
      const path = mdFile.path;
      if (path) {
        await openFile(path);
      } else {
        // 在浏览器中运行时，读取文件内容并调用渲染
        const reader = new FileReader();
        reader.onload = async (evt) => {
          const content = evt.target.result;
          try {
            const { invoke } = window.__TAURI__.core;
            const result = await invoke('render_markdown', { content });
            // 设置状态
            state.currentFile = mdFile.name;
            state.currentHtml = result.html;
            state.metadata = result.metadata;
            document.getElementById('file-title').textContent = mdFile.name;
            document.title = `${mdFile.name} — Zephyr`;
            document.getElementById('welcome').classList.add('hidden');
            const contentEl = document.getElementById('content');
            contentEl.classList.remove('hidden');
            contentEl.innerHTML = result.html;
          } catch (err) {
            console.error('渲染失败:', err);
          }
        };
        reader.readAsText(mdFile);
      }
    }
  });
}

function setupFileWatcher() {
  if (window.__TAURI__) {
    const { listen } = window.__TAURI__.event;
    listen('file-changed', async (event) => {
      const path = event.payload;
      if (path === state.currentFile) {
        // 自动重新加载（在 Tauri 中，使用 confirm 会阻塞）
        console.log('文件已修改，自动重新加载:', path);
        await openFile(path);
      }
    });
  }
}

function handleCliArgs() {
  // 检查从 Rust 端传递的 CLI 参数
  if (window.__ZEPHYR_CLI_FILE__) {
    // 延迟到页面完全加载后打开
    setTimeout(async () => {
      const path = window.__ZEPHYR_CLI_FILE__;
      if (path) {
        await openFile(path);
      }
    }, 500);
  }
}

// 等待 DOM 加载
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
