import state from '../state.js';

export function initSearch() {
  const toggleBtn = document.getElementById('toggle-search');
  const searchBar = document.getElementById('search-bar');
  const input = document.getElementById('search-input');
  const prevBtn = document.getElementById('search-prev');
  const nextBtn = document.getElementById('search-next');
  const closeBtn = document.getElementById('search-close');
  const countEl = document.getElementById('search-count');

  toggleBtn.addEventListener('click', () => {
    searchBar.classList.toggle('hidden');
    if (!searchBar.classList.contains('hidden')) {
      input.focus();
      input.select();
    } else {
      clearHighlights();
      countEl.textContent = '';
    }
  });

  // 键盘快捷键
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
      e.preventDefault();
      searchBar.classList.toggle('hidden');
      if (!searchBar.classList.contains('hidden')) {
        input.focus();
        input.select();
      }
    }
    if (e.key === 'Escape' && !searchBar.classList.contains('hidden')) {
      searchBar.classList.add('hidden');
      clearHighlights();
      countEl.textContent = '';
    }
    if (e.key === 'Enter' && !searchBar.classList.contains('hidden')) {
      e.preventDefault();
      if (e.shiftKey) {
        navigateSearch(-1);
      } else {
        navigateSearch(1);
      }
    }
  });

  let debounceTimer;
  input.addEventListener('input', () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => performSearch(input.value), 250);
  });

  prevBtn.addEventListener('click', () => navigateSearch(-1));
  nextBtn.addEventListener('click', () => navigateSearch(1));
  closeBtn.addEventListener('click', () => {
    searchBar.classList.add('hidden');
    clearHighlights();
    countEl.textContent = '';
    input.value = '';
  });
}

function performSearch(query) {
  const countEl = document.getElementById('search-count');
  clearHighlights();

  if (!query || !state.currentHtml) {
    countEl.textContent = '';
    return;
  }

  const container = document.getElementById('content');
  if (!container) {
    countEl.textContent = '';
    return;
  }

  // 使用 TreeWalker 在文本节点中搜索（忽略 HTML 标签）
  const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null, false);
  const matches = [];
  const nodes = [];

  while (walker.nextNode()) {
    nodes.push(walker.currentNode);
  }

  const lowerQuery = query.toLowerCase();

  nodes.forEach(node => {
    const text = node.textContent;
    let pos = 0;
    while ((pos = text.toLowerCase().indexOf(lowerQuery, pos)) !== -1) {
      matches.push({ node, offset: pos, length: query.length });
      pos += query.length;
    }
  });

  state.searchResults = matches;
  state.searchIndex = 0;

  if (matches.length === 0) {
    countEl.textContent = '无匹配';
    return;
  }

  countEl.textContent = `${matches.length} 个匹配`;

  // 高亮所有匹配
  // 注意：surroundContents 会修改 DOM，需要从后往前处理以避免偏移问题
  matches.forEach((match, idx) => {
    try {
      const range = document.createRange();
      range.setStart(match.node, match.offset);
      range.setEnd(match.node, match.offset + match.length);
      const span = document.createElement('span');
      span.className = 'search-highlight';
      if (idx === 0) span.classList.add('active');
      range.surroundContents(span);
    } catch (e) {
      // 跳过重叠范围（部分已处理）
    }
  });

  // 滚动到第一个匹配
  const firstActive = document.querySelector('.search-highlight.active');
  if (firstActive) {
    firstActive.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

function navigateSearch(direction) {
  const results = state.searchResults;
  if (!results || results.length === 0) return;

  // 移除当前 active 状态
  const currentActive = document.querySelector('.search-highlight.active');
  if (currentActive) currentActive.classList.remove('active');

  // 计算新的索引
  state.searchIndex = (state.searchIndex + direction + results.length) % results.length;

  // 高亮当前匹配
  const highlights = document.querySelectorAll('.search-highlight');
  if (highlights.length > 0 && highlights[state.searchIndex]) {
    highlights[state.searchIndex].classList.add('active');
    highlights[state.searchIndex].scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

function clearHighlights() {
  const highlights = document.querySelectorAll('.search-highlight');
  highlights.forEach(el => {
    const parent = el.parentNode;
    if (parent) {
      const text = document.createTextNode(el.textContent);
      parent.replaceChild(text, el);
      parent.normalize();
    }
  });
  state.searchResults = [];
  state.searchIndex = 0;
}
