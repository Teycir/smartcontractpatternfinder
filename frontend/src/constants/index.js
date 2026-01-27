/**
 * Application-wide constants
 * Centralized configuration for timeouts, API endpoints, and default values
 */

// API Configuration
export const API_ENDPOINTS = {
  STATUS: '/api/status',
  START: '/api/start',
  PAUSE: '/api/pause',
  RESUME: '/api/resume',
  STOP: '/api/stop',
  LOGS: '/api/logs',
  HEALTH: '/api/health',
}

// Timeout Configuration (in milliseconds)
export const TIMEOUTS = {
  API_STATUS: 3000,
  API_ACTION: 5000,
  API_START: 10000,
  ERROR_DISPLAY: 8000,
  STATUS_POLL_INTERVAL: 2000,
  LOG_STATUS_POLL_INTERVAL: 1500,
  CONNECTION_TIMEOUT: 5000,
  RECONNECT_DELAY_BASE: 5000,
  RECONNECT_DELAY_MAX: 30000,
  SUCCESS_DISPLAY: 5000,
  COPY_FEEDBACK: 2000,
}

// Connection Configuration
export const CONNECTION_CONFIG = {
  MAX_RECONNECT_ATTEMPTS: 5,
  MAX_LOG_ENTRIES: 1000,
}

// Initial States
export const INITIAL_PROGRESS = {
  contracts_scanned: 0,
  contracts_total: null,
  current_contract: null,
  current_contract_name: null,
  contracts_extracted: 0,
  eta_seconds: 0,
  rate: null,
  critical_findings: 0,
}

export const INITIAL_CONFIG = {
  addresses: '',
  chain: 'all',
  pages: 5,
  concurrency: 2,
  tags: '',
  contract_type: '',
  extract_sources: '50',
  fetch_zero_day: true,
}

// Scan Presets
export const SCAN_PRESETS = {
  quick: {
    pages: 3,
    chain: 'ethereum',
    concurrency: 5,
    tags: '',
    contract_type: '',
    extract_sources: '25',
    fetch_zero_day: false,
  },
  deep: {
    pages: 10,
    chain: 'all',
    concurrency: 3,
    tags: '',
    contract_type: '',
    extract_sources: '100',
    fetch_zero_day: true,
  },
  zeroday: {
    pages: 0,
    chain: 'all',
    concurrency: 2,
    tags: '',
    contract_type: '',
    extract_sources: '0',
    fetch_zero_day: true,
  },
}

// Validation Rules
export const VALIDATION = {
  PAGES_MIN: 0,
  PAGES_MAX: 100,
  CONCURRENCY_MIN: 1,
  CONCURRENCY_MAX: 20,
}

// Chain Configuration
export const CHAINS = {
  ALL: 'all',
  ALL_EXPANDED: 'ethereum,polygon,arbitrum',
}

// Status Types
export const SCAN_STATUS = {
  IDLE: 'idle',
  RUNNING: 'running',
  PAUSED: 'paused',
  STOPPED: 'stopped',
}

export const CONNECTION_STATUS = {
  DISCONNECTED: 'disconnected',
  CONNECTING: 'connecting',
  CONNECTED: 'connected',
  ERROR: 'error',
}
