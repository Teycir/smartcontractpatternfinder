/**
 * Validation utilities
 * Centralized validation logic to avoid duplication
 */

import { VALIDATION } from '../constants'

/**
 * Validate days value
 * @param {string|number} value - The days value to validate
 * @returns {{ isValid: boolean, error: string|null }}
 */
export const validateDays = (value) => {
  const days = parseInt(value, 10)
  if (isNaN(days) || days < VALIDATION.DAYS_MIN) {
    return { isValid: false, error: 'Must be 0 or greater' }
  }
  if (days > VALIDATION.DAYS_MAX) {
    return { isValid: false, error: `Must be ${VALIDATION.DAYS_MAX} or less` }
  }
  return { isValid: true, error: null }
}

/**
 * Validate concurrency value
 * @param {string|number} value - The concurrency value to validate
 * @returns {{ isValid: boolean, error: string|null }}
 */
export const validateConcurrency = (value) => {
  const concurrency = parseInt(value, 10)
  if (isNaN(concurrency) || concurrency < VALIDATION.CONCURRENCY_MIN || concurrency > VALIDATION.CONCURRENCY_MAX) {
    return { isValid: false, error: `Must be between ${VALIDATION.CONCURRENCY_MIN}-${VALIDATION.CONCURRENCY_MAX}` }
  }
  return { isValid: true, error: null }
}

/**
 * Parse addresses string to array
 * @param {string} addressString - Comma-separated addresses
 * @returns {string[]}
 */
export const parseAddresses = (addressString) => {
  return addressString
    .split(',')
    .map(a => a.trim())
    .filter(a => a.length > 0)
}

/**
 * Build API payload from config
 * @param {object} config - The configuration object
 * @returns {object}
 */
export const buildScanPayload = (config) => {
  const days = parseInt(config.days, 10)
  const concurrency = parseInt(config.concurrency, 10)
  const extractSources = parseInt(config.extract_sources, 10)

  return {
    addresses: parseAddresses(config.addresses),
    chain: config.chain === 'all' ? 'ethereum,polygon,arbitrum' : config.chain,
    days,
    concurrency,
    tags: config.tags || null,
    contract_type: config.contract_type || null,
    extract_sources: extractSources > 0 ? extractSources : null,
    fetch_zero_day: config.fetch_zero_day ? 30 : null,
  }
}
