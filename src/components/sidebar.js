import state from '../state.js';
import { openFile } from './viewer.js';

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
  document.getElementById('toggle-sidebar').textContent = state.sidebarVisible ? '✕' : '☰';
}

function switchTab(tab) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.sidebar-panel').forEach(p => p.classList.remove('active'));

  document.querySelector(`.tab-btn[data-tab="${tab}"]`).classList.add('active');
  document.getElementById(`${tab}-panel`).classList.add('active');

  // 切换到文件标签时自动刷新文件树
  if (tab === 'files' && state.currentFile) {
    refreshFileTree();
  }
}

function openFileDialog() {
  const input = document.getElementById('file-input');
  input.click();
  input.onchange = async () => {
    const file = input.files[0];
    if (file) {
      // 需要文件路径来调用 Rust 命令
      // 拖拽会提供路径，但文件选择器只提供 File 对象（不含路径）
      // 对于 Tauri，需要通过 dialog 插件或拖拽
      // fallback: 使用 Tauri API
      openFileViaTauriDialog();
    }
    input.value = '';
  };
}

async function openFileViaTauriDialog() {
  try {
    const { open } = await import('https://cdn.jsdelivr.net/npm/@tauri-apps/plugin-dialog@2/+esm');
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Markdown', extensions: ['md', 'markdown'] }],
    });
    if (selected) {
      await openFile(selected);
    }
  } catch (e) {
    console.warn('Tauri dialog 不可用，尝试使用前端文件读取作为回退', e);
    // 回退：使用 FileReader 读取最近选择的文件（仅当通过拖拽获得路径时有效）
  }
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
      el.innerHTML = `<span class="icon">📁</span>${escapeHtml(entry.name)}`;
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
      el.innerHTML = `<span class="icon">📄</span>${escapeHtml(entry.name)}`;
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
    // 只在文件标签页可见时更新
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
