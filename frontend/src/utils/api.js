/**
 * API utilities
 * Centralized API calls with error handling
 */

import axios from 'axios'
import { API_ENDPOINTS, TIMEOUTS } from '../constants'
import { loadRuntimeConfig } from './runtimeConfig'

let apiPromise = null

const getApiClient = async () => {
  if (!apiPromise) {
    apiPromise = loadRuntimeConfig()
      .then(({ apiBaseUrl }) =>
        axios.create({
          baseURL: apiBaseUrl,
        })
      )
      .catch((error) => {
        apiPromise = null
        throw error
      })
  }

  return apiPromise
}

const apiRequest = async (config) => {
  const api = await getApiClient()
  return api.request(config)
}

const SCPF_SERVICE_ID = 'scpf-server'

const assertScpfService = (data, fallbackMessage) => {
  if (data?.service !== SCPF_SERVICE_ID) {
    throw new Error(fallbackMessage)
  }
  return data
}

/**
 * Extract error message from axios error
 * @param {Error} error - The error object
 * @param {string} fallback - Fallback message
 * @returns {string}
 */
export const getErrorMessage = (error, fallback = 'An error occurred') => {
  if (typeof error.response?.data === 'string' && error.response.data.trim().length > 0) {
    return error.response.data
  }
  return error.response?.data?.error || error.message || fallback
}

/**
 * Fetch scan status
 * @returns {Promise<object>}
 */
export const fetchScanStatus = async () => {
  const response = await apiRequest({
    method: 'get',
    url: API_ENDPOINTS.STATUS,
    timeout: TIMEOUTS.API_STATUS 
  })
  return assertScpfService(
    response.data,
    'Unexpected backend response. Another local service may be bound to the configured SCPF port.'
  )
}

/**
 * Start a scan
 * @param {object} payload - Scan configuration
 * @returns {Promise<object>}
 */
export const startScan = async (payload) => {
  const response = await apiRequest({
    method: 'post',
    url: API_ENDPOINTS.START,
    data: payload,
    timeout: TIMEOUTS.API_START 
  })
  return response.data
}

/**
 * Pause the current scan
 * @returns {Promise<object>}
 */
export const pauseScan = async () => {
  const response = await apiRequest({
    method: 'post',
    url: API_ENDPOINTS.PAUSE,
    data: {},
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Resume the paused scan
 * @returns {Promise<object>}
 */
export const resumeScan = async () => {
  const response = await apiRequest({
    method: 'post',
    url: API_ENDPOINTS.RESUME,
    data: {},
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Stop the current scan
 * @returns {Promise<object>}
 */
export const stopScan = async () => {
  const response = await apiRequest({
    method: 'post',
    url: API_ENDPOINTS.STOP,
    data: {},
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Check server health
 * @returns {Promise<object>}
 */
export const checkHealth = async () => {
  const response = await apiRequest({
    method: 'get',
    url: API_ENDPOINTS.HEALTH,
    timeout: TIMEOUTS.API_STATUS 
  })
  return assertScpfService(
    response.data,
    'Unexpected backend health response. Another local service may be bound to the configured SCPF port.'
  )
}

export const fetchTemplates = async () => {
  const response = await apiRequest({
    method: 'get',
    url: API_ENDPOINTS.TEMPLATES,
    timeout: TIMEOUTS.API_STATUS 
  })
  const data = assertScpfService(
    response.data,
    'Unexpected template response. Another local service may be bound to the configured SCPF port.'
  )

  if (!Array.isArray(data.templates)) {
    throw new Error('Template inventory response was malformed.')
  }

  return data
}

export const exportLogs = async (logs) => {
  const response = await apiRequest({
    method: 'post',
    url: API_ENDPOINTS.EXPORT_LOGS,
    data: { logs },
    timeout: TIMEOUTS.API_ACTION,
  })
  return response.data
}

export const getApiUrl = async (path = '') => {
  const { apiBaseUrl } = await loadRuntimeConfig()
  const base = apiBaseUrl.endsWith('/') ? apiBaseUrl : `${apiBaseUrl}/`
  const normalizedPath = path.startsWith('/') ? path.slice(1) : path
  return new URL(normalizedPath, base).toString()
}
