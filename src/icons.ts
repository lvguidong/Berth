const paths: Record<string, string> = {
  folder:
    '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  anchor:
    '<circle cx="12" cy="5" r="2"/><path d="M12 7v14M8 11h8M4 15c0 8 16 8 16 0M2 17l2-2 2 2m12 0 2-2 2 2"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  settings:
    '<path d="m9 3-1 3-3 1v4l-2 1 2 2v4l3 1 1 3h5l1-3 3-1v-4l2-2-2-1V7l-3-1-1-3z"/><circle cx="11.5" cy="12.5" r="3"/>',
  search: '<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>',
  pin: '<path d="m8 3 8 0-1 7 3 3v2H6v-2l3-3zM12 15v6"/>',
  more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
  close: '<path d="m6 6 12 12M6 18 18 6"/>',
  arrow: '<path d="m9 5 7 7-7 7"/>',
  undo: '<path d="M4 10h10a6 6 0 1 1 0 12M8 5l-5 5 5 5"/>',
  window:
    '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 9h18"/>',
  check: '<path d="m5 12 4 4L19 6"/>',
  grip: '<path d="M9 5h.01M15 5h.01M9 12h.01M15 12h.01M9 19h.01M15 19h.01" stroke-width="3"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1 1m12 12 1 1M5 19l1-1M18 6l1-1"/>',
};
export const icon = (name: string) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths[name] || paths.folder}</svg>`;
