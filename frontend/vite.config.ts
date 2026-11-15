import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// In development the API runs on :8080; in Docker nginx proxies /api instead.
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          react: ['react', 'react-dom', 'react-router-dom', '@tanstack/react-query'],
          mantine: ['@mantine/core', '@mantine/hooks', '@mantine/notifications'],
          charts: ['@mantine/charts', 'recharts'],
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: { '/api': process.env.VITE_API_PROXY ?? 'http://localhost:8080' },
  },
});
