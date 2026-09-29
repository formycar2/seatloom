/// <reference types="vitest/config" />
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// Vitest config for SeatLoom UI. jsdom environment so component tests can mount
// React trees. No test files exist yet (v0.1); this bootstraps the runner so
// `pnpm test` is green (zero tests = pass) and new tests have a home.
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    passWithNoTests: true,
  },
});
