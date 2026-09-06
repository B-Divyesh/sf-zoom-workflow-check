const reset = document.querySelector('#reset-demo');

reset?.addEventListener('click', () => {
  window.location.assign('/demo/');
});

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => navigator.serviceWorker.register('/sw.js'));
}
