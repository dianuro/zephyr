import state from '../state.js';
import { openFile } from './viewer.js';

const FOLDER_SVG = `<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" stroke="none">
  <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h3l1.5 1.5h5A1.5 1.5 0 0 1 13.5 5v6a1.5 1.5 0 0 1-1.5 1.5h-10A1.5 1.5 0 0 1 .5 10V5.5z"/>
</svg>`;

const FILE_SVG = `<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
  <path d="M2.5.5h5l4 4v9a1 1 0 0 1-1 1h-8a1 1 0 0 1-1-1v-12a1 1 0 0 1 1-1z"/>
  <polyline points="7.5.5 7.5 4.5 11.5 4.5"/>
</svg>`;

export function initSidebar() {
  document.getElementById('toggle-sidebar').addEventListener('click', toggleSidebar);
  document.getElementById('open-file').addEventListener('click', openFileDialog);

  document.querySelectorAll('.tab-btn').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });
}

export function toggleSidebar() {
  state.sidebarVisible = !state.sidebarVisible;
  document.getElementById('sidebar').classList.toggle('hidden', !state.sidebarVisible);
}

function switchTab(tab) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.sidebar-panel').forEach(p => p.classList.remove('active'));

  document.querySelector(`.tab-btn[data-tab="${tab}"]`).classList.add('active');
  document.getElementById(`${tab}-panel`).classList.add('active');

  if (tab === 'files' && state.currentFile) {
    refreshFileTree();
  }
}

async function openFileDialog() {
  try {
    const { invoke } = window.__TAURI__.core;
    const path = await invoke('select_markdown_file');
    if (path) {
      await openFile(path);
    }
  } catch (e) {
    console.error('打开文件对话框失败:', e);
    // 回退：尝试使用 HTML 文件输入
    fallbackFileInput();
  }
}

function fallbackFileInput() {
  // 在标准浏览器环境中，通过 FileReader 提供基础支持
  const input = document.getElementById('file-input');
  if (!input) return;

  input.value = '';
  input.click();
  input.onchange = async () => {
    const file = input.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = async (evt) => {
        const content = evt.target?.result;
        if (typeof content === 'string') {
          try {
            const { invoke } = window.__TAURI__.core;
            const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
            const result = await invoke('render_markdown', { content, isDark });
            state.currentFile = file.name;
            state.currentHtml = result.html;
            state.metadata = result.metadata;
            document.getElementById('file-title').textContent = file.name;
            document.title = `${file.name} — Zephyr`;
            document.getElementById('welcome').classList.add('hidden');
            const contentEl = document.getElementById('content');
            contentEl.classList.remove('hidden');
            contentEl.innerHTML = result.html;
          } catch (err) {
            console.error('渲染失败:', err);
          }
        }
      };
      reader.readAsText(file);
    }
    input.value = '';
  };
}

export function renderFileTree(tree) {
  const container = document.getElementById('file-tree');
  container.innerHTML = '';

  if (!tree || !tree.entries || tree.entries.length === 0) {
    container.innerHTML = '<div style="padding: 16px; color: var(--text-muted); font-size: 13px;">空目录</div>';
    return;
  }

  tree.entries.forEach(entry => {
    const el = document.createElement('div');
    el.className = 'file-item';
    el.dataset.path = entry.path;

    if (entry.is_dir) {
      el.classList.add('dir');
      el.innerHTML = `<span class="file-icon folder-icon">${FOLDER_SVG}</span>${escapeHtml(entry.name)}`;
      el.addEventListener('click', async () => {
        try {
          const { invoke } = window.__TAURI__.core;
          const newTree = await invoke('get_file_tree', { path: entry.path });
          renderFileTree(newTree);
        } catch (e) {
          console.warn('读取目录失败:', e);
        }
      });
    } else if (entry.is_markdown) {
      el.innerHTML = `<span class="file-icon">${FILE_SVG}</span>${escapeHtml(entry.name)}`;
      el.addEventListener('click', () => openFile(entry.path));
    }

    container.appendChild(el);
  });
}

async function refreshFileTree() {
  if (!state.currentFile) return;
  try {
    const { invoke } = window.__TAURI__.core;
    const tree = await invoke('get_file_tree', { path: state.currentFile });
    state.fileTree = tree;
    if (document.querySelector('.tab-btn[data-tab="files"].active')) {
      renderFileTree(tree);
    }
  } catch (e) {
    console.warn('刷新文件树失败:', e);
  }
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
