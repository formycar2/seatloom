import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// Vite config for the SeatLoom UI.
// Port 1420 is the conventional Tauri dev-server port; tauri.conf.json
// references it via `build.devUrl`. Strict port so a port clash fails fast
// rather than silently moving the webview target.
//
// Multi-entry: the main window loads index.html; the detached Supervisor
// window loads supervisor.html. Both are served by the same Vite dev server
// and bundled into dist/ in production.
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, 'index.html'),
        supervisor: path.resolve(__dirname, 'supervisor.html'),
      },
    },
  },
})
