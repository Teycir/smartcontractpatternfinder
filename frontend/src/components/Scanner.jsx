import React, { useState, useEffect, useRef, useCallback } from 'react'
import axios from 'axios'
import Console from './Console'
import './Scanner.css'

const Scanner = () => {
  const [status, setStatus] = useState('idle')
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [serverOnline, setServerOnline] = useState(false)
  const [progress, setProgress] = useState({
    contracts_scanned: 0,
    contracts_total: null,
    current_contract: null,
    vulnerabilities_found: 0,
  })
  const [config, setConfig] = useState({
    addresses: '',
    chain: 'all',
    days: 100,
    concurrency: 3,
    no_cache: false,
    tags: '',
    contract_type: '',
    sort_by_exploitability: false,
    update_templates: '0',
    extract_sources: '0',
  })

  const statusIntervalRef = useRef(null)
  const errorTimeoutRef = useRef(null)

  const showError = useCallback((message) => {
    setError(message)
    if (errorTimeoutRef.current) {
      clearTimeout(errorTimeoutRef.current)
    }
    errorTimeoutRef.current = setTimeout(() => {
      setError('')
    }, 8000)
  }, [])

  const fetchStatus = useCallback(async () => {
    try {
      const response = await axios.get('/api/status', { timeout: 3000 })
      setServerOnline(true)
      setStatus(response.data.status || 'idle')

      // Update progress
      if (response.data.progress) {
        setProgress(response.data.progress)
      }

      if (response.data.config) {
        const cfg = response.data.config
        setConfig(prev => ({
          ...prev,
          addresses: cfg.addresses?.join(', ') || prev.addresses,
          chain: cfg.chain || prev.chain,
          days: cfg.days ?? prev.days,
          concurrency: cfg.concurrency ?? prev.concurrency,
          no_cache: cfg.no_cache ?? prev.no_cache,
          tags: cfg.tags || prev.tags,
          contract_type: cfg.contract_type || prev.contract_type,
          sort_by_exploitability: cfg.sort_by_exploitability ?? prev.sort_by_exploitability,
          update_templates: cfg.update_templates != null ? cfg.update_templates.toString() : prev.update_templates,
          extract_sources: cfg.extract_sources != null ? cfg.extract_sources.toString() : prev.extract_sources,
        }))
      }
    } catch (err) {
      setServerOnline(false)
      // Only log network errors once to avoid console spam
      if (err.code !== 'ECONNABORTED') {
        console.debug('Status fetch failed:', err.message)
      }
    }
  }, [])

  useEffect(() => {
    fetchStatus()
    statusIntervalRef.current = setInterval(fetchStatus, 2000)

    return () => {
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current)
      }
      if (errorTimeoutRef.current) {
        clearTimeout(errorTimeoutRef.current)
      }
    }
  }, [fetchStatus])

  const handleStart = async () => {
    if (isLoading) return

    setIsLoading(true)
    setError('')

    try {
      // Validate configuration
      const days = parseInt(config.days, 10)
      const concurrency = parseInt(config.concurrency, 10)

      if (isNaN(days) || days < 1) {
        throw new Error('Days must be a positive number')
      }

      if (isNaN(concurrency) || concurrency < 1 || concurrency > 20) {
        throw new Error('Concurrency must be between 1 and 20')
      }

      // Parse update_templates carefully to avoid NaN
      let updateTemplatesValue = null
      if (config.update_templates && config.update_templates !== '0') {
        const parsed = parseInt(config.update_templates, 10)
        if (!isNaN(parsed) && parsed > 0) {
          updateTemplatesValue = parsed
        }
      }

      const payload = {
        addresses: config.addresses
          .split(',')
          .map(a => a.trim())
          .filter(a => a.length > 0),
        chain: config.chain === 'all' ? 'ethereum,polygon,arbitrum' : config.chain,
        days,
        concurrency,
        no_cache: Boolean(config.no_cache),
        tags: config.tags || null,
        contract_type: config.contract_type || null,
        sort_by_exploitability: Boolean(config.sort_by_exploitability),
        update_templates: updateTemplatesValue,
        extract_sources: config.extract_sources && parseInt(config.extract_sources, 10) > 0
          ? parseInt(config.extract_sources, 10)
          : null,
      }

      await axios.post('/api/start', payload, { timeout: 10000 })
      setStatus('running')
    } catch (err) {
      const errorMessage = err.response?.data?.error || err.message || 'Failed to start scan'
      showError(errorMessage)
      console.error('Start scan error:', err)
    } finally {
      setIsLoading(false)
    }
  }

  const handlePause = async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      await axios.post('/api/pause', {}, { timeout: 5000 })
      setStatus('paused')
    } catch (err) {
      const errorMessage = err.response?.data?.error || err.message || 'Failed to pause scan'
      showError(errorMessage)
    } finally {
      setIsLoading(false)
    }
  }

  const handleResume = async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      await axios.post('/api/resume', {}, { timeout: 5000 })
      setStatus('running')
    } catch (err) {
      const errorMessage = err.response?.data?.error || err.message || 'Failed to resume scan'
      showError(errorMessage)
    } finally {
      setIsLoading(false)
    }
  }

  const handleStop = async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      await axios.post('/api/stop', {}, { timeout: 5000 })
      // Backend resets to 'idle', sync with that
      setStatus('idle')
      // Reset progress display
      setProgress({
        contracts_scanned: 0,
        contracts_total: null,
        current_contract: null,
        vulnerabilities_found: 0,
      })
    } catch (err) {
      const errorMessage = err.response?.data?.error || err.message || 'Failed to stop scan'
      showError(errorMessage)
    } finally {
      setIsLoading(false)
    }
  }

  const handleInputChange = (e) => {
    const { name, value, type, checked } = e.target
    setConfig(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }))
  }

  const getStatusIcon = () => {
    if (!serverOnline) return '⚫'
    switch (status) {
      case 'running': return '🟢'
      case 'paused': return '🟡'
      case 'stopped': return '🔴'
      default: return '⚪'
    }
  }

  const getStatusText = () => {
    if (!serverOnline) return 'OFFLINE'
    return status.toUpperCase()
  }

  const isControlsDisabled = status === 'running' || isLoading

  return (
    <div className="scanner">
      {error && (
        <div
          className="error-toast"
          onClick={() => setError('')}
          role="alert"
          style={{
            position: 'fixed',
            top: '20px',
            right: '20px',
            background: '#ef4444',
            color: 'white',
            padding: '1rem 1.5rem',
            borderRadius: '8px',
            boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
            zIndex: 1000,
            maxWidth: '400px',
            cursor: 'pointer',
            animation: 'slideIn 0.3s ease-out'
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span>❌</span>
            <span>{error}</span>
          </div>
          <div style={{ fontSize: '0.75rem', marginTop: '0.5rem', opacity: 0.8 }}>
            Click to dismiss
          </div>
        </div>
      )}

      <div className="scanner-controls">
        <div className="status-bar">
          <span className={`status-indicator status-${status}`}>
            {getStatusIcon()}
          </span>
          <span className="status-text">
            Status: <strong>{getStatusText()}</strong>
            {!serverOnline && <span style={{ color: '#f85149', marginLeft: '0.5rem' }}>(Server not responding)</span>}
          </span>
        </div>

        <div className="control-buttons">
          <button
            onClick={handleStart}
            disabled={status === 'running' || status === 'paused' || isLoading || !serverOnline}
            className="btn btn-start"
            title={!serverOnline ? 'Server is offline' : status === 'running' ? 'Scan in progress' : 'Start scan'}
          >
            {isLoading && status !== 'running' ? '⏳' : '▶️'} Start
          </button>
          <button
            onClick={status === 'paused' ? handleResume : handlePause}
            disabled={(status !== 'running' && status !== 'paused') || isLoading}
            className={`btn ${status === 'paused' ? 'btn-resume' : 'btn-pause'}`}
          >
            {status === 'paused' ? '▶️ Resume' : '⏸️ Pause'}
          </button>
          <button
            onClick={handleStop}
            disabled={(status !== 'running' && status !== 'paused') || isLoading}
            className="btn btn-stop"
          >
            ⏹️ Stop
          </button>
        </div>
      </div>

      {/* Progress Bar */}
      {(status === 'running' || status === 'paused') && (
        <div className="progress-section">
          <div className="progress-header">
            <span className="progress-label">
              📊 Progress: {progress.contracts_scanned}
              {progress.contracts_total ? ` / ${progress.contracts_total}` : ''} contracts scanned
            </span>
            <span className="progress-vulns">
              🔴 {progress.vulnerabilities_found} vulnerabilities found
            </span>
          </div>
          {progress.contracts_total && progress.contracts_total > 0 && (
            <div className="progress-bar-container">
              <div
                className="progress-bar-fill"
                style={{ width: `${Math.min(100, (progress.contracts_scanned / progress.contracts_total) * 100)}%` }}
              />
            </div>
          )}
          {progress.current_contract && (
            <div className="progress-current">
              Current: <code>{progress.current_contract}</code>
            </div>
          )}
        </div>
      )}

      <div className="scanner-config">
        <h2>Configuration</h2>

        <div className="config-grid">
          <div className="config-group">
            <label>Contract Addresses <span className="label-hint">(empty = autodetect)</span></label>
            <input
              type="text"
              name="addresses"
              value={config.addresses}
              onChange={handleInputChange}
              placeholder="0x1234..., 0x5678..."
              disabled={isControlsDisabled}
            />
          </div>

          <div className="config-group">
            <label>Blockchain Chain</label>
            <select name="chain" value={config.chain} onChange={handleInputChange} disabled={isControlsDisabled}>
              <option value="all">All (Ethereum, Polygon, Arbitrum)</option>
              <option value="ethereum">Ethereum</option>
              <option value="polygon">Polygon</option>
              <option value="arbitrum">Arbitrum</option>
            </select>
          </div>

          <div className="config-group">
            <label>Days to Scan</label>
            <input
              type="number"
              name="days"
              value={config.days}
              onChange={handleInputChange}
              min="1"
              max="365"
              disabled={isControlsDisabled}
            />
          </div>

          <div className="config-group">
            <label>Concurrency</label>
            <input
              type="number"
              name="concurrency"
              value={config.concurrency}
              onChange={handleInputChange}
              min="1"
              max="20"
              disabled={isControlsDisabled}
            />
          </div>

          <div className="config-group">
            <label>Contract Type</label>
            <select name="contract_type" value={config.contract_type} onChange={handleInputChange} disabled={isControlsDisabled}>
              <option value="">All Types</option>
              <option value="erc20">ERC-20</option>
              <option value="erc721">ERC-721</option>
              <option value="erc1155">ERC-1155</option>
              <option value="proxy">Proxy</option>
              <option value="defi">DeFi</option>
            </select>
          </div>

          <div className="config-group">
            <label>Tags (filter templates)</label>
            <select name="tags" value={config.tags} onChange={handleInputChange} disabled={isControlsDisabled}>
              <option value="">All Templates</option>
              <option value="security">Security</option>
              <option value="reentrancy">Reentrancy</option>
              <option value="access-control">Access Control</option>
              <option value="defi">DeFi</option>
            </select>
          </div>

          <div className="config-group">
            <label>Update Templates</label>
            <select name="update_templates" value={config.update_templates} onChange={handleInputChange} disabled={isControlsDisabled}>
              <option value="0">No Update</option>
              <option value="1">1 Day</option>
              <option value="7">7 Days</option>
              <option value="30">30 Days</option>
            </select>
          </div>
        </div>

        <div className="config-checkboxes">
          <label className="checkbox-label">
            <input
              type="checkbox"
              name="no_cache"
              checked={config.no_cache}
              onChange={handleInputChange}
              disabled={isControlsDisabled}
            />
            <span>No Cache (fetch fresh data)</span>
          </label>

          <label className="checkbox-label">
            <input
              type="checkbox"
              name="sort_by_exploitability"
              checked={config.sort_by_exploitability}
              onChange={handleInputChange}
              disabled={isControlsDisabled}
            />
            <span>Sort by Exploitability</span>
          </label>
        </div>

        <div className="config-group" style={{ marginTop: '1rem' }}>
          <label>Extract Top Riskiest Sources</label>
          <select name="extract_sources" value={config.extract_sources} onChange={handleInputChange} disabled={isControlsDisabled}>
            <option value="0">Don't Extract</option>
            <option value="10">Top 10</option>
            <option value="25">Top 25</option>
            <option value="50">Top 50</option>
            <option value="100">Top 100</option>
            <option value="200">Top 200</option>
          </select>
        </div>
      </div>

      <Console />

      <style>{`
        @keyframes slideIn {
          from {
            opacity: 0;
            transform: translateX(100%);
          }
          to {
            opacity: 1;
            transform: translateX(0);
          }
        }
      `}</style>
    </div>
  )
}

export default Scanner
