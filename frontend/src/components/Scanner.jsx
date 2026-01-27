import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import Console from './Console'
import './Scanner.css'
import {
  TIMEOUTS,
  INITIAL_PROGRESS,
  INITIAL_CONFIG,
  SCAN_PRESETS,
  SCAN_STATUS,
} from '../constants'
import { validatePages, validateConcurrency, buildScanPayload } from '../utils/validation'
import { fetchScanStatus, startScan, pauseScan, resumeScan, stopScan, getErrorMessage } from '../utils/api'

/**
 * Scanner component - Main interface for configuring and running scans
 */
const Scanner = () => {
  // State management
  const [status, setStatus] = useState(SCAN_STATUS.IDLE)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [serverOnline, setServerOnline] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)
  const [validationErrors, setValidationErrors] = useState({})
  const [progress, setProgress] = useState(INITIAL_PROGRESS)
  const [config, setConfig] = useState(INITIAL_CONFIG)
  const [userStopped, setUserStopped] = useState(false)

  // Refs for intervals and cleanup
  const statusIntervalRef = useRef(null)
  const errorTimeoutRef = useRef(null)
  const prevStatusRef = useRef(SCAN_STATUS.IDLE)

  // ===== CALLBACKS =====

  /**
   * Display an error toast that auto-dismisses
   */
  const showError = useCallback((message) => {
    setError(message)
    if (errorTimeoutRef.current) {
      clearTimeout(errorTimeoutRef.current)
    }
    errorTimeoutRef.current = setTimeout(() => {
      setError('')
    }, TIMEOUTS.ERROR_DISPLAY)
  }, [])

  /**
   * Clear a specific validation error
   */
  const clearValidationError = useCallback((field) => {
    setValidationErrors(prev => {
      const { [field]: removed, ...rest } = prev
      return rest
    })
  }, [])

  /**
   * Poll the backend for current status
   */
  const pollStatus = useCallback(async () => {
    try {
      const data = await fetchScanStatus()
      setServerOnline(true)
      setStatus(data.status || SCAN_STATUS.IDLE)

      if (data.progress) {
        console.log('Progress data received:', data.progress)
        setProgress(data.progress)
      }

      // Sync config from server when running
      if (data.config && status !== SCAN_STATUS.IDLE) {
        const cfg = data.config
        setConfig(prev => ({
          ...prev,
          addresses: cfg.addresses?.join(', ') || prev.addresses,
          chain: cfg.chain || prev.chain,
          pages: cfg.pages ?? prev.pages,
          concurrency: cfg.concurrency ?? prev.concurrency,
          tags: cfg.tags || prev.tags,
          contract_type: cfg.contract_type || prev.contract_type,
          extract_sources: cfg.extract_sources != null ? cfg.extract_sources.toString() : prev.extract_sources,
          fetch_zero_day: cfg.fetch_zero_day ?? prev.fetch_zero_day,
        }))
      }
    } catch (err) {
      setServerOnline(false)
      if (err.code !== 'ECONNABORTED') {
        console.debug('Status fetch failed:', err.message)
      }
    }
  }, [status])

  // ===== ACTION HANDLERS =====

  const handleStart = useCallback(async () => {
    if (isLoading) return

    setIsLoading(true)
    setError('')
    setValidationErrors({})
    setUserStopped(false)

    try {
      const pagesValidation = validatePages(config.pages)
      if (!pagesValidation.isValid) {
        throw new Error(pagesValidation.error)
      }

      const concurrencyValidation = validateConcurrency(config.concurrency)
      if (!concurrencyValidation.isValid) {
        throw new Error(concurrencyValidation.error)
      }

      const payload = buildScanPayload(config)
      await startScan(payload)
      setStatus(SCAN_STATUS.RUNNING)
    } catch (err) {
      showError(getErrorMessage(err, 'Failed to start scan'))
    } finally {
      setIsLoading(false)
    }
  }, [isLoading, config, showError])

  const handlePause = useCallback(async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      await pauseScan()
      setStatus(SCAN_STATUS.PAUSED)
    } catch (err) {
      showError(getErrorMessage(err, 'Failed to pause scan'))
    } finally {
      setIsLoading(false)
    }
  }, [isLoading, showError])

  const handleResume = useCallback(async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      await resumeScan()
      setStatus(SCAN_STATUS.RUNNING)
    } catch (err) {
      showError(getErrorMessage(err, 'Failed to resume scan'))
    } finally {
      setIsLoading(false)
    }
  }, [isLoading, showError])

  const handleStop = useCallback(async () => {
    if (isLoading) return

    setIsLoading(true)
    setUserStopped(true)
    try {
      await stopScan()
      setStatus(SCAN_STATUS.IDLE)
      setProgress(INITIAL_PROGRESS)
    } catch (err) {
      showError(getErrorMessage(err, 'Failed to stop scan'))
    } finally {
      setIsLoading(false)
    }
  }, [isLoading, showError])

  /**
   * Handle form input changes with inline validation
   */
  const handleInputChange = useCallback((e) => {
    const { name, value, type, checked } = e.target
    const newValue = type === 'checkbox' ? checked : value
    
    setConfig(prev => ({ ...prev, [name]: newValue }))

    // Inline validation for specific fields
    if (name === 'pages') {
      const validation = validatePages(value)
      if (!validation.isValid) {
        setValidationErrors(prev => ({ ...prev, pages: validation.error }))
      } else {
        clearValidationError('pages')
      }
    } else if (name === 'concurrency') {
      const validation = validateConcurrency(value)
      if (!validation.isValid) {
        setValidationErrors(prev => ({ ...prev, concurrency: validation.error }))
      } else {
        clearValidationError('concurrency')
      }
    }
  }, [clearValidationError])

  /**
   * Apply a configuration preset
   */
  const applyPreset = useCallback((presetName) => {
    const preset = SCAN_PRESETS[presetName]
    if (preset) {
      setConfig(prev => ({ ...prev, ...preset }))
      setValidationErrors({})
    }
  }, [])

  // ===== EFFECTS =====

  // Status polling
  useEffect(() => {
    pollStatus()
    statusIntervalRef.current = setInterval(pollStatus, TIMEOUTS.STATUS_POLL_INTERVAL)

    return () => {
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current)
      }
      if (errorTimeoutRef.current) {
        clearTimeout(errorTimeoutRef.current)
      }
    }
  }, [pollStatus])

  // Track previous status
  useEffect(() => {
    prevStatusRef.current = status
  }, [status])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyPress = (e) => {
      const isTyping = ['INPUT', 'SELECT', 'TEXTAREA'].includes(e.target.tagName)
      if (isTyping) return

      if (e.code === 'Space') {
        e.preventDefault()
        if (status === SCAN_STATUS.IDLE) handleStart()
        else if (status === SCAN_STATUS.RUNNING) handlePause()
        else if (status === SCAN_STATUS.PAUSED) handleResume()
      } else if (e.code === 'Escape') {
        if (status === SCAN_STATUS.RUNNING || status === SCAN_STATUS.PAUSED) {
          e.preventDefault()
          handleStop()
        }
      }
    }

    window.addEventListener('keydown', handleKeyPress)
    return () => window.removeEventListener('keydown', handleKeyPress)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status])

  // ===== COMPUTED VALUES =====

  const statusIcon = useMemo(() => {
    if (!serverOnline) return '⚫'
    switch (status) {
      case SCAN_STATUS.RUNNING: return '🟢'
      case SCAN_STATUS.PAUSED: return '🟡'
      case SCAN_STATUS.STOPPED: return '🔴'
      default: return '⚪'
    }
  }, [serverOnline, status])

  const statusText = useMemo(() => {
    if (!serverOnline) return 'OFFLINE'
    return status.toUpperCase()
  }, [serverOnline, status])

  const isControlsDisabled = status === SCAN_STATUS.RUNNING || isLoading

  const progressPercent = useMemo(() => {
    if (!progress.contracts_total || progress.contracts_total === 0) return 0
    return Math.min(100, (progress.contracts_scanned / progress.contracts_total) * 100)
  }, [progress.contracts_scanned, progress.contracts_total])

  const etaDisplay = useMemo(() => {
    const eta = progress.eta_seconds
    if (!eta || isNaN(eta) || eta <= 0) return null
    const mins = Math.floor(eta / 60)
    const secs = Math.floor(eta % 60)
    return `${mins}m${secs}s`
  }, [progress.eta_seconds])

  const shouldShowProgress = status === SCAN_STATUS.RUNNING || 
    status === SCAN_STATUS.PAUSED || 
    (progress.contracts_scanned > 0 && progress.contracts_total > 0)

  const progressBarColor = useMemo(() => {
    if (status === SCAN_STATUS.PAUSED) return '#f59e0b'
    if (status === SCAN_STATUS.IDLE) return '#6b7280'
    return undefined
  }, [status])

  const isZeroDayMode = config.pages === 0 || config.pages === '0'

  // ===== RENDER =====

  return (
    <div className="scanner">
      {/* Error Toast */}
      {error && (
        <div
          className="error-toast"
          onClick={() => setError('')}
          role="alert"
          aria-live="assertive"
        >
          <div className="error-toast-content">
            <span>❌</span>
            <span>{error}</span>
          </div>
          <div className="error-toast-hint">Click to dismiss</div>
        </div>
      )}



      {/* Scanner Controls */}
      <div className="scanner-controls">
        <div className="status-bar">
          <span className={`status-indicator status-${status} ${status === SCAN_STATUS.RUNNING ? 'pulse-indicator' : ''}`}>
            {statusIcon}
          </span>
          <span className="status-text">
            Status: <strong>{statusText}</strong>
            {!serverOnline && (
              <span className="status-offline">(Server not responding)</span>
            )}
          </span>
        </div>

        <div className="control-buttons">
          <button
            onClick={handleStart}
            disabled={status === SCAN_STATUS.RUNNING || status === SCAN_STATUS.PAUSED || isLoading || !serverOnline}
            className="btn btn-start"
            title={!serverOnline ? 'Server is offline' : status === SCAN_STATUS.RUNNING ? 'Scan in progress' : 'Start scan (Space)'}
          >
            {isLoading && status !== SCAN_STATUS.RUNNING ? '⏳' : '▶️'} Start
          </button>
          <button
            onClick={status === SCAN_STATUS.PAUSED ? handleResume : handlePause}
            disabled={(status !== SCAN_STATUS.RUNNING && status !== SCAN_STATUS.PAUSED) || isLoading}
            className={`btn ${status === SCAN_STATUS.PAUSED ? 'btn-resume' : 'btn-pause'}`}
            title={status === SCAN_STATUS.PAUSED ? 'Resume (Space)' : 'Pause (Space)'}
          >
            {status === SCAN_STATUS.PAUSED ? '▶️ Resume' : '⏸️ Pause'}
          </button>
          <button
            onClick={handleStop}
            disabled={(status !== SCAN_STATUS.RUNNING && status !== SCAN_STATUS.PAUSED) || isLoading}
            className="btn btn-stop"
            title="Stop scan (Esc)"
          >
            ⏹️ Stop
          </button>
        </div>
      </div>

      {/* Progress Section */}
      {shouldShowProgress && (
        <div className="progress-section">
          <div className="progress-header">
            <span className="progress-label">
              📊 {status === SCAN_STATUS.IDLE && progress.contracts_scanned > 0 && progress.contracts_scanned >= progress.contracts_total ? 'All' : progress.contracts_scanned}
              {progress.contracts_total && !(status === SCAN_STATUS.IDLE && progress.contracts_scanned >= progress.contracts_total) ? ` / ${progress.contracts_total}` : ''} contracts scanned
              {progress.contracts_total > 0 && !(status === SCAN_STATUS.IDLE && progress.contracts_scanned >= progress.contracts_total) && (
                <strong className="progress-percent">({progressPercent.toFixed(1)}%)</strong>
              )}
              {etaDisplay && (
                <span className="progress-eta">• ETA: {etaDisplay}</span>
              )}
              {progress.rate && (
                <span className="progress-rate">• {progress.rate.toFixed(1)}/s</span>
              )}
              {status === SCAN_STATUS.IDLE && progress.contracts_extracted > 0 && !userStopped && (
                <span className="progress-complete">✅ Complete</span>
              )}
            </span>
          </div>
          {progress.contracts_total > 0 && (
            <div className="progress-bar-container">
              <div
                className="progress-bar-fill"
                style={{ 
                  width: `${progressPercent}%`,
                  backgroundColor: progressBarColor,
                }}
                role="progressbar"
                aria-valuenow={progressPercent}
                aria-valuemin="0"
                aria-valuemax="100"
              />
            </div>
          )}
          {progress.current_contract_name && status === SCAN_STATUS.RUNNING && (
            <div className="progress-current">
              🔄 Currently scanning: <code>{progress.current_contract_name}</code>
            </div>
          )}
        </div>
      )}

      {/* Configuration Section */}
      <div className="scanner-config">
        <div className="config-header">
          <h2>Configuration</h2>
          <div className="preset-buttons">
            <button 
              className="btn-preset" 
              onClick={() => applyPreset('quick')}
              disabled={isControlsDisabled}
              title="Quick scan: 5 pages, All chains, Fast"
            >
              🚀 Quick
            </button>
            <button 
              className="btn-preset" 
              onClick={() => applyPreset('deep')}
              disabled={isControlsDisabled}
              title="Deep scan: 50 pages, All chains, Full analysis"
            >
              🔍 Deep
            </button>
            <button 
              className="btn-preset" 
              onClick={() => applyPreset('zeroday')}
              disabled={isControlsDisabled}
              title="0-Day only: Skip scanning, fetch recent exploits"
            >
              ⚡ 0-Day
            </button>
          </div>
        </div>

        <div className="config-grid">
          <div className="config-group">
            <label htmlFor="addresses">
              Contract Addresses <span className="label-hint">(empty = autodetect)</span>
            </label>
            <input
              id="addresses"
              type="text"
              name="addresses"
              value={config.addresses}
              onChange={handleInputChange}
              placeholder="0x1234..., 0x5678..."
              disabled={isControlsDisabled}
              title="Leave empty to auto-detect contracts from recent transactions"
            />
          </div>

          <div className="config-group">
            <label htmlFor="chain">Blockchain Chain</label>
            <select 
              id="chain"
              name="chain" 
              value={config.chain} 
              onChange={handleInputChange} 
              disabled={isControlsDisabled}
              title="Select which blockchain networks to scan"
            >
              <option value="all">All (Ethereum, Polygon, Arbitrum)</option>
              <option value="ethereum">Ethereum</option>
              <option value="polygon">Polygon</option>
              <option value="arbitrum">Arbitrum</option>
            </select>
          </div>

          <div className="config-group">
            <label htmlFor="pages">
              Pages to Fetch{' '}
              {isZeroDayMode && (
                <span className="label-hint">(no scan, only 0-day reports)</span>
              )}
            </label>
            <input
              id="pages"
              type="number"
              name="pages"
              value={config.pages}
              onChange={handleInputChange}
              min="0"
              max="100"
              disabled={isControlsDisabled}
              className={validationErrors.pages ? 'input-error' : ''}
              title="Number of pages to fetch (100 contracts per page). Set to 0 for only 0-day reports without scanning"
              aria-invalid={!!validationErrors.pages}
              aria-describedby={validationErrors.pages ? 'pages-error' : undefined}
            />
            {validationErrors.pages && (
              <span id="pages-error" className="validation-error" role="alert">
                {validationErrors.pages}
              </span>
            )}
          </div>

          <div className="config-group">
            <label htmlFor="contract_type">Contract Type</label>
            <select 
              id="contract_type"
              name="contract_type" 
              value={config.contract_type} 
              onChange={handleInputChange} 
              disabled={isControlsDisabled}
            >
              <option value="">All Types</option>
              <option value="erc20">ERC-20</option>
              <option value="erc721">ERC-721</option>
              <option value="erc1155">ERC-1155</option>
              <option value="proxy">Proxy</option>
              <option value="defi">DeFi</option>
            </select>
          </div>

          <div className="config-group">
            <label htmlFor="tags">Tags (filter templates)</label>
            <select 
              id="tags"
              name="tags" 
              value={config.tags} 
              onChange={handleInputChange} 
              disabled={isControlsDisabled}
            >
              <option value="">All Templates</option>
              <option value="security">Security</option>
              <option value="reentrancy">Reentrancy</option>
              <option value="access-control">Access Control</option>
              <option value="defi">DeFi</option>
            </select>
          </div>
        </div>

        <div className="config-checkboxes">
          <label className="checkbox-label" title="Fetch and analyze recently disclosed vulnerabilities">
            <input
              type="checkbox"
              name="fetch_zero_day"
              checked={config.fetch_zero_day}
              onChange={handleInputChange}
              disabled={isControlsDisabled}
            />
            <span>Fetch 0-day exploits <small className="checkbox-hint">(last 30 days)</small></span>
          </label>
        </div>

        <div className="config-group config-group-bottom">
          <label htmlFor="extract_sources">Extract Top Riskiest Sources</label>
          <select 
            id="extract_sources"
            name="extract_sources" 
            value={config.extract_sources} 
            onChange={handleInputChange} 
            disabled={isControlsDisabled}
            title="Number of highest-risk contract sources to extract for detailed review"
          >
            <option value="10">Top 10</option>
            <option value="25">Top 25</option>
            <option value="50">Top 50 (Default)</option>
            <option value="100">Top 100</option>
            <option value="200">Top 200</option>
            <option value="0">Don't Extract</option>
          </select>
        </div>
      </div>

      <Console />
    </div>
  )
}

export default Scanner
