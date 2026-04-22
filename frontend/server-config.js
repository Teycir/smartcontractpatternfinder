const DEFAULT_SERVER_ADDR = '127.0.0.1:32145'

export const SCPF_SERVER_ADDR = (process.env.SCPF_SERVER_ADDR || DEFAULT_SERVER_ADDR)
  .replace(/^https?:\/\//, '')
  .replace(/\/$/, '')

export const SCPF_SERVER_ORIGIN = process.env.VITE_API_URL || `http://${SCPF_SERVER_ADDR}`
