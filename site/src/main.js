const copyStatus = document.querySelector('#copy-status');

document.querySelectorAll('[data-copy]').forEach((button) => {
  button.addEventListener('click', async () => {
    const text = document.getElementById(button.dataset.copy)?.textContent ?? '';
    try {
      await navigator.clipboard.writeText(text);
      button.firstChild.textContent = 'Copied';
      copyStatus.textContent = 'Install command copied.';
      window.setTimeout(() => { button.firstChild.textContent = 'Copy'; }, 1800);
    } catch {
      copyStatus.textContent = 'Copy was blocked. Select the command and copy it manually.';
    }
  });
});

const tabs = [...document.querySelectorAll('[role="tab"][data-zoom]')];
const panel = document.querySelector('#panel-zoom');
const demo = {
  result: document.querySelector('#demo-result'),
  viewport: document.querySelector('#demo-viewport'),
  obscured: document.querySelector('#demo-obscured'),
  scroll: document.querySelector('#demo-scroll'),
  detail: document.querySelector('#demo-detail'),
  visual: document.querySelector('.viewport-demo'),
  proof: document.querySelector('.browser-proof')
};

function selectZoom(next) {
  for (const tab of tabs) {
    const active = tab === next;
    tab.setAttribute('aria-selected', String(active));
    tab.tabIndex = active ? 0 : -1;
  }
  const zoom = next.dataset.zoom;
  panel.setAttribute('aria-labelledby', next.id);
  demo.visual.dataset.state = zoom;
  demo.proof.setAttribute('aria-label', `Illustrated browser viewport at ${zoom} percent zoom`);
  if (zoom === '400') {
    demo.result.textContent = 'Blocked at 400%';
    demo.viewport.textContent = '320 × 225 CSS px';
    demo.obscured.textContent = 'Yes — flyout edge';
    demo.scroll.textContent = 'Unavailable';
    demo.detail.textContent = 'The payment control is clipped by a fixed-height flyout, and the flyout cannot scroll to reveal it.';
  } else {
    demo.result.textContent = 'Passes at 200%';
    demo.viewport.textContent = '640 × 450 CSS px';
    demo.obscured.textContent = 'No';
    demo.scroll.textContent = 'Available';
    demo.detail.textContent = 'The focused payment button remains fully inside the flyout and viewport.';
  }
}

for (const tab of tabs) {
  tab.addEventListener('click', () => selectZoom(tab));
  tab.addEventListener('keydown', (event) => {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const current = tabs.indexOf(tab);
    const index = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : event.key === 'ArrowRight' ? (current + 1) % tabs.length : (current - 1 + tabs.length) % tabs.length;
    tabs[index].focus();
    selectZoom(tabs[index]);
  });
}

const offlineNote = document.querySelector('#offline-note');
function updateConnection() { offlineNote.hidden = navigator.onLine; }
window.addEventListener('online', updateConnection);
window.addEventListener('offline', updateConnection);
updateConnection();

if ('serviceWorker' in navigator) window.addEventListener('load', () => navigator.serviceWorker.register('/sw.js'));
