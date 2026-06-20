import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Stellar SDK expects a `global` in some code paths; map it to globalThis for the browser.
export default defineConfig({
  plugins: [react()],
  define: {
    global: 'globalThis',
  },
});
