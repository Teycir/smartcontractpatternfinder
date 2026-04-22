import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { SCPF_SERVER_ORIGIN } from './server-config'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: process.env.VITE_API_URL || SCPF_SERVER_ORIGIN,
        changeOrigin: true,
      },
    },
  },
})
