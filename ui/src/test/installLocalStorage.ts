// A jsdom localStorage that actually works.
//
// vitest's jsdom environment does NOT provide localStorage (the Node webstorage
// stub is an empty object with no methods), so any test that mounts a component
// whose store uses zustand's `persist` crashes on `storage.setItem is not a
// function` the moment it calls setState. Importing this module fixes that for
// the whole file.
//
// Import it FIRST in a test file: module evaluation follows import order, and
// zustand binds `localStorage` when the store is CREATED, so the double has to
// exist before the store module is imported.
const backing: Record<string, string> = {};

Object.defineProperty(window, 'localStorage', {
  configurable: true,
  value: {
    getItem: (k: string) => (k in backing ? backing[k] : null),
    setItem: (k: string, v: string) => {
      backing[k] = String(v);
    },
    removeItem: (k: string) => {
      delete backing[k];
    },
    clear: () => {
      for (const k of Object.keys(backing)) delete backing[k];
    },
    key: (i: number) => Object.keys(backing)[i] ?? null,
    get length() {
      return Object.keys(backing).length;
    },
  },
});
