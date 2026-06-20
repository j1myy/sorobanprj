import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

// @stellar/stellar-sdk references Node globals (Buffer/process/global) in the
// browser; nodePolyfills shims them so the app doesn't white-screen at runtime.
export default defineConfig({
  plugins: [
    react(),
    nodePolyfills({
      globals: { Buffer: true, global: true, process: true },
    }),
  ],
});
