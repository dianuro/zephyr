import state from '../state.js';
import { openFile } from './viewer.js';

// ====== 图标 SVG ======
const FOLDER_SVG = `<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" stroke="none">
  <path d="M1 3.5A1.5 1.5 0 0 1 2.5 2h3l1.5 1.5h5A1.5 1.5 0 0 1 13.5 5v6a1.5 1.5 0 0 1-1.5 1.5h-10A1.5 1.5 0 0 1 .5 10V5.5z"/>
</svg>`;

const FILE_SVG = `<svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
  <path d="M2.5.5h5l4 4v9a1 1 0 0 1-1 1h-8a1 1 0 0 1-1-1v-12a1 1 0 0 1 1-1z"/>
  <polyline points="7.5.5 7.5 4.5 11.5 4.5"/>
</svg>`;

const CHEVRON_RIGHT = `<svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="4.5,3 7.5,6 4.5,9"/></svg>`;

const CHEVRON_DOWN = `<svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="3,4.5 6,7.5 9,4.5"/></svg>`;

// ====== 树节点存储 ======
// node: { key, name, path, isDir, expanded, children (array of keys), loaded }
const nodes = new Map();

// ====== 初始化 ======
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
}

// ====== 文件对话框 ======
async function openFileDialog() {
  try {
    const { invoke } = window.__TAURI__.core;
    const path = await invoke('select_markdown_file');
    if (path) {
      await openFile(path);
    }
  } catch (e) {
    console.error('打开文件对话框失败:', e);
    fallbackFileInput();
  }
}

function fallbackFileInput() {
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

// ====== 文件树 ======

/** 以 rootPath 为根重建树 */
export async function buildFileTree(rootPath) {
  nodes.clear();
  const key = await ensureChildren(rootPath, null);
  renderTree();
  return key;
}

/** 确保节点已加载子节点，返回节点 key */
async function ensureChildren(path, parentKey) {
  const key = pathToKey(path);
  if (nodes.has(key) && nodes.get(key).loaded) return key;

  try {
    const { invoke } = window.__TAURI__.core;
    const tree = await invoke('get_file_tree', { path });
    const entries = tree.entries || [];

    // 创建或更新节点
    if (!nodes.has(key)) {
      // 从路径推断名称
      const parts = path.replace(/\\/g, '/').split('/').filter(Boolean);
      const name = parts[parts.length - 1] || path;
      nodes.set(key, {
        key,
        name,
        path,
        isDir: true,
        expanded: parentKey === null, // 根节点默认展开
        children: [],
        loaded: false,
        parentKey,
      });
    }

    const node = nodes.get(key);
    node.children = [];
    node.loaded = true;

    for (const entry of entries) {
      const childKey = pathToKey(entry.path);
      if (!nodes.has(childKey)) {
        nodes.set(childKey, {
          key: childKey,
          name: entry.name,
          path: entry.path,
          isDir: entry.is_dir,
          expanded: false,
          children: null,
          loaded: false,
          parentKey: key,
        });
      }
      node.children.push(childKey);
    }

    return key;
  } catch (e) {
    console.warn('读取目录失败:', e);
    return key;
  }
}

function pathToKey(p) {
  // 统一路径分隔符
  return p.replace(/\\/g, '/');
}

/** 切换目录展开/折叠 */
async function toggleDir(key) {
  const node = nodes.get(key);
  if (!node || !node.isDir) return;

  if (!node.loaded) {
    await ensureChildren(node.path, node.parentKey);
  }

  node.expanded = !node.expanded;
  renderTree();
}

/** 渲染整棵树 */
function renderTree() {
  const container = document.getElementById('file-tree');
  container.innerHTML = '';

  // 找到根节点（parentKey 为 null 的节点）
  const roots = [];
  for (const [, node] of nodes) {
    if (node.parentKey === null) {
      roots.push(node.key);
    }
  }

  if (roots.length === 0) {
    container.innerHTML = '<div class="tree-empty">空目录</div>';
    return;
  }

  for (const rootKey of roots) {
    renderNode(rootKey, 0, container);
  }
}

/** 递归渲染一个节点及其子节点 */
function renderNode(key, depth, parentEl) {
  const node = nodes.get(key);
  if (!node) return;

  const item = document.createElement('div');
  item.className = 'tree-item';
  item.dataset.path = node.path;
  if (node.path === state.currentFile) {
    item.classList.add('active');
  }

  // 缩进
  item.style.paddingLeft = `${8 + depth * 16}px`;

  // 箭头（仅目录）
  if (node.isDir) {
    const arrow = document.createElement('span');
    arrow.className = 'tree-arrow';
    arrow.innerHTML = node.children && node.children.length > 0
      ? (node.expanded ? CHEVRON_DOWN : CHEVRON_RIGHT)
      : '<span class="tree-arrow-spacer"></span>';
    item.appendChild(arrow);

    arrow.addEventListener('click', (e) => {
      e.stopPropagation();
      toggleDir(key);
    });
  } else {
    // 文件缩进占位
    const spacer = document.createElement('span');
    spacer.className = 'tree-arrow-spacer';
    spacer.style.width = '16px';
    spacer.style.display = 'inline-block';
    item.appendChild(spacer);
  }

  // 图标
  const icon = document.createElement('span');
  icon.className = 'tree-icon';
  icon.innerHTML = node.isDir ? FOLDER_SVG : FILE_SVG;
  item.appendChild(icon);

  // 名称
  const nameSpan = document.createElement('span');
  nameSpan.className = 'tree-name';
  nameSpan.textContent = node.name;
  item.appendChild(nameSpan);

  // 点击行为
  item.addEventListener('click', () => {
    if (node.isDir) {
      toggleDir(key);
    } else {
      openFile(node.path);
    }
  });

  parentEl.appendChild(item);

  // 递归展开子节点
  if (node.isDir && node.expanded && node.children) {
    for (const childKey of node.children) {
      renderNode(childKey, depth + 1, parentEl);
    }
  }
}
