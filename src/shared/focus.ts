// Run `callback` whenever the person comes back to the window (focus or tab becomes visible); returns the cleanup.
// Data that only changes outside the Hub (an installer, a command line) is read again at that moment instead of on a timer.
export function onReturn(callback: () => void): () => void {
  const visible = () => { if (document.visibilityState === 'visible') callback(); };
  window.addEventListener('focus', callback);
  document.addEventListener('visibilitychange', visible);
  return () => { window.removeEventListener('focus', callback); document.removeEventListener('visibilitychange', visible); };
}
