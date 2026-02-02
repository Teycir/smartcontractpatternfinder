/**
 * API utilities
 * Centralized API calls with error handling
 */

import axios from 'axios'
import { API_ENDPOINTS, API_BASE_URL, TIMEOUTS } from '../constants'

const api = axios.create({
  baseURL: API_BASE_URL,
})

/**
 * Extract error message from axios error
 * @param {Error} error - The error object
 * @param {string} fallback - Fallback message
 * @returns {string}
 */
export const getErrorMessage = (error, fallback = 'An error occurred') => {
  return error.response?.data?.error || error.message || fallback
}

/**
 * Fetch scan status
 * @returns {Promise<object>}
 */
export const fetchScanStatus = async () => {
  const response = await api.get(API_ENDPOINTS.STATUS, { 
    timeout: TIMEOUTS.API_STATUS 
  })
  return response.data
}

/**
 * Start a scan
 * @param {object} payload - Scan configuration
 * @returns {Promise<object>}
 */
export const startScan = async (payload) => {
  const response = await api.post(API_ENDPOINTS.START, payload, { 
    timeout: TIMEOUTS.API_START 
  })
  return response.data
}

/**
 * Pause the current scan
 * @returns {Promise<object>}
 */
export const pauseScan = async () => {
  const response = await api.post(API_ENDPOINTS.PAUSE, {}, { 
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Resume the paused scan
 * @returns {Promise<object>}
 */
export const resumeScan = async () => {
  const response = await api.post(API_ENDPOINTS.RESUME, {}, { 
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Stop the current scan
 * @returns {Promise<object>}
 */
export const stopScan = async () => {
  const response = await api.post(API_ENDPOINTS.STOP, {}, { 
    timeout: TIMEOUTS.API_ACTION 
  })
  return response.data
}

/**
 * Check server health
 * @returns {Promise<object>}
 */
export const checkHealth = async () => {
  const response = await api.get(API_ENDPOINTS.HEALTH, { 
    timeout: TIMEOUTS.API_STATUS 
  })
  return response.data
}

export const fetchTemplates = async () => {
  const response = await api.get(API_ENDPOINTS.TEMPLATES, { 
    timeout: TIMEOUTS.API_STATUS 
  })
  return response.data
}
