/**
 * Validation utilities
 * Centralized validation logic to avoid duplication
 */

import { VALIDATION } from '../constants'

/**
 * Validate pages value
 * @param {string|number} value - The pages value to validate
 * @returns {{ isValid: boolean, error: string|null }}
 */
export const validatePages = (value) => {
  const pages = parseInt(value, 10)
  if (isNaN(pages) || pages < VALIDATION.PAGES_MIN) {
    return { isValid: false, error: 'Must be 0 or greater' }
  }
  if (pages > VALIDATION.PAGES_MAX) {
    return { isValid: false, error: `Must be ${VALIDATION.PAGES_MAX} or less` }
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
    .split(/[\n,]/)
    .map(a => a.trim())
    .filter(a => a.length > 0)
}

/**
 * Build API payload from config
 * @param {object} config - The configuration object
 * @returns {object}
 */
export const buildScanPayload = (config) => {
  const pages = parseInt(config.pages, 10)
  const concurrency = parseInt(config.concurrency, 10)
  const extractSources = parseInt(config.extract_sources, 10)

  if (isNaN(pages) || isNaN(concurrency)) {
    throw new Error('Invalid numeric values in configuration')
  }

  return {
    addresses: parseAddresses(config.addresses),
    chain: config.chain === 'all' ? 'ethereum,polygon,arbitrum' : config.chain,
    pages,
    concurrency,
    tags: config.tags || null,
    contract_type: config.contract_type || null,
    extract_sources: !isNaN(extractSources) && extractSources > 0 ? extractSources : null,
    fetch_zero_day: config.fetch_zero_day ? 30 : null,
  }
}
