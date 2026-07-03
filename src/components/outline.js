export function renderOutline(headings) {
  const container = document.getElementById('outline-tree');
  container.innerHTML = '';

  if (!headings || headings.length === 0) {
    container.innerHTML = '<div style="padding: 16px; color: var(--text-muted); font-size: 13px;">无标题结构</div>';
    return;
  }

  headings.forEach(h => renderHeadingItem(h, container));
}

function renderHeadingItem(heading, parent) {
  const el = document.createElement('a');
  el.className = `outline-item h${Math.min(heading.level, 4)}`;
  el.textContent = heading.text || '(无标题)';
  el.href = `#${heading.id}`;
  el.addEventListener('click', (e) => {
    e.preventDefault();
    const target = document.getElementById(heading.id);
    if (target) {
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  });
  parent.appendChild(el);

  if (heading.children && heading.children.length > 0) {
    heading.children.forEach(child => renderHeadingItem(child, parent));
  }
}
