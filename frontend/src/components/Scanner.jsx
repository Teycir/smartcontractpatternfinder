import React, { useState, useEffect, useRef, useCallback } from 'react'
import axios from 'axios'
import Console from './Console'
import './Scanner.css'

const Scanner = () => {
  const [status, setStatus] = useState('idle')
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [serverOnline, setServerOnline] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)
  const [validationErrors, setValidationErrors] = useState({})
  const [progress, setProgress] = useState({
    contracts_scanned: 0,
    contracts_total: null,
    current_contract: null,
    current_contract_name: null,
    contracts_extracted: 0,
    eta_seconds: 0,
    rate: null,
    critical_findings: 0,
  })
  const [config, setConfig] = useState({
    addresses: '',
    chain: 'all',
    days: 100,
    concurrency: 2,
    tags: '',
    contract_type: '',
    extract_sources: '50',
    fetch_zero_day: true,
  })

  const statusIntervalRef = useRef(null)
  const errorTimeoutRef = useRef(null)
  const prevStatusRef = useRef('idle')

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

      if (response.data.config && status !== 'idle') {
        const cfg = response.data.config
        setConfig(prev => ({
          ...prev,
          addresses: cfg.addresses?.join(', ') || prev.addresses,
          chain: cfg.chain || prev.chain,
          days: cfg.days ?? prev.days,
          concurrency: cfg.concurrency ?? prev.concurrency,
          tags: cfg.tags || prev.tags,
          contract_type: cfg.contract_type || prev.contract_type,
          extract_sources: cfg.extract_sources != null ? cfg.extract_sources.toString() : prev.extract_sources,
          fetch_zero_day: cfg.fetch_zero_day ?? prev.fetch_zero_day,
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

  // Detect scan completion and show success animation
  useEffect(() => {
    if (prevStatusRef.current === 'running' && status === 'idle' && 
        progress.contracts_scanned > 0 && progress.contracts_scanned >= progress.contracts_total) {
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 5000)
    }
    prevStatusRef.current = status
  }, [status, progress])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyPress = (e) => {
      // Ignore if typing in an input field
      if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') {
        return
      }

      if (e.code === 'Space') {
        e.preventDefault()
        if (status === 'idle') {
          handleStart()
        } else if (status === 'running') {
          handlePause()
        } else if (status === 'paused') {
          handleResume()
        }
      } else if (e.code === 'Escape' && (status === 'running' || status === 'paused')) {
        e.preventDefault()
        handleStop()
      }
    }

    window.addEventListener('keydown', handleKeyPress)
    return () => window.removeEventListener('keydown', handleKeyPress)
  }, [status])

  const handleStart = async () => {
    if (isLoading) return

    setIsLoading(true)
    setError('')

    try {
      // Validate configuration
      const days = parseInt(config.days, 10)
      const concurrency = parseInt(config.concurrency, 10)

      if (isNaN(days) || days < 0) {
        throw new Error('Days must be 0 or greater (0 = only 0-day reports, no scanning)')
      }

      if (isNaN(concurrency) || concurrency < 1 || concurrency > 20) {
        throw new Error('Concurrency must be between 1 and 20')
      }

      const payload = {
        addresses: config.addresses
          .split(',')
          .map(a => a.trim())
          .filter(a => a.length > 0),
        chain: config.chain === 'all' ? 'ethereum,polygon,arbitrum' : config.chain,
        days,
        concurrency,
        tags: config.tags || null,
        contract_type: config.contract_type || null,
        extract_sources: config.extract_sources && parseInt(config.extract_sources, 10) > 0
          ? parseInt(config.extract_sources, 10)
          : null,
        fetch_zero_day: config.fetch_zero_day ? 30 : null,
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
        current_contract_name: null,
        contracts_extracted: 0,
        eta_seconds: 0,
        rate: null,
        critical_findings: 0,
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
    const newValue = type === 'checkbox' ? checked : value
    
    setConfig(prev => ({
      ...prev,
      [name]: newValue
    }))

    // Inline validation
    if (name === 'days') {
      const days = parseInt(value, 10)
      if (isNaN(days) || days < 0) {
        setValidationErrors(prev => ({ ...prev, days: 'Must be 0 or greater' }))
      } else {
        setValidationErrors(prev => {
          const { days, ...rest } = prev
          return rest
        })
      }
    } else if (name === 'concurrency') {
      const concurrency = parseInt(value, 10)
      if (isNaN(concurrency) || concurrency < 1 || concurrency > 20) {
        setValidationErrors(prev => ({ ...prev, concurrency: 'Must be between 1-20' }))
      } else {
        setValidationErrors(prev => {
          const { concurrency, ...rest } = prev
          return rest
        })
      }
    }
  }

  const applyPreset = (preset) => {
    const presets = {
      quick: {
        days: 7,
        chain: 'ethereum',
        concurrency: 5,
        tags: '',
        contract_type: '',
        extract_sources: '25',
        fetch_zero_day: false,
      },
      deep: {
        days: 30,
        chain: 'all',
        concurrency: 3,
        tags: '',
        contract_type: '',
        extract_sources: '100',
        fetch_zero_day: true,
      },
      zeroday: {
        days: 0,
        chain: 'all',
        concurrency: 2,
        tags: '',
        contract_type: '',
        extract_sources: '0',
        fetch_zero_day: true,
      },
    }
    
    if (presets[preset]) {
      setConfig(prev => ({ ...prev, ...presets[preset] }))
      setValidationErrors({})
    }
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

      {showSuccess && (
        <div className="success-celebration">
          <div className="celebration-content">
            <span className="celebration-icon">🎉</span>
            <span className="celebration-text">Scan Complete!</span>
            <span className="celebration-icon">✨</span>
          </div>
        </div>
      )}

      <div className="scanner-controls">
        <div className="status-bar">
          <span className={`status-indicator status-${status} ${status === 'running' ? 'pulse-indicator' : ''}`}>
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
            title={!serverOnline ? 'Server is offline' : status === 'running' ? 'Scan in progress' : 'Start scan (Space)'}
          >
            {isLoading && status !== 'running' ? '⏳' : '▶️'} Start
          </button>
          <button
            onClick={status === 'paused' ? handleResume : handlePause}
            disabled={(status !== 'running' && status !== 'paused') || isLoading}
            className={`btn ${status === 'paused' ? 'btn-resume' : 'btn-pause'}`}
            title={status === 'paused' ? 'Resume (Space)' : 'Pause (Space)'}
          >
            {status === 'paused' ? '▶️ Resume' : '⏸️ Pause'}
          </button>
          <button
            onClick={handleStop}
            disabled={(status !== 'running' && status !== 'paused') || isLoading}
            className="btn btn-stop"
            title="Stop scan (Esc)"
          >
            ⏹️ Stop
          </button>
        </div>
      </div>

      {/* Progress Bar - show when running, paused, or has progress data */}
      {(status === 'running' || status === 'paused' || (progress.contracts_scanned > 0 && progress.contracts_total > 0)) && (
        <div className="progress-section">
          <div className="progress-header">
            <span className="progress-label">
              📊 {progress.contracts_scanned}
              {progress.contracts_total ? ` / ${progress.contracts_total}` : ''} contracts scanned
              {progress.contracts_total > 0 && (
                <strong style={{ marginLeft: '0.5rem' }}>
                  ({((progress.contracts_scanned / progress.contracts_total) * 100).toFixed(1)}%)
                </strong>
              )}
              {progress.eta_seconds > 0 && (
                <span style={{ marginLeft: '0.5rem', color: '#3b82f6' }}>
                  • ETA: {Math.floor(progress.eta_seconds / 60)}m{progress.eta_seconds % 60}s
                </span>
              )}
              {progress.rate && (
                <span style={{ marginLeft: '0.5rem', color: '#10b981' }}>
                  • {progress.rate.toFixed(1)}/s
                </span>
              )}
              {progress.critical_findings > 0 && (
                <span style={{ marginLeft: '0.5rem', color: '#ef4444' }}>
                  • 🚨 {progress.critical_findings} critical
                </span>
              )}
              {progress.contracts_extracted > 0 && (
                <span style={{ marginLeft: '0.5rem' }}>
                  • {progress.contracts_extracted} extracted
                </span>
              )}
              {status === 'idle' && progress.contracts_scanned > 0 && progress.contracts_scanned < progress.contracts_total && (
                <span style={{ color: '#f59e0b', marginLeft: '0.5rem' }}>⚠️ Stopped</span>
              )}
              {status === 'idle' && progress.contracts_scanned > 0 && progress.contracts_scanned >= progress.contracts_total && (
                <span style={{ color: '#22c55e', marginLeft: '0.5rem' }}>✅ Complete</span>
              )}
            </span>
          </div>
          {progress.contracts_total && progress.contracts_total > 0 && (
            <div className="progress-bar-container">
              <div
                className="progress-bar-fill"
                style={{ 
                  width: `${Math.min(100, (progress.contracts_scanned / progress.contracts_total) * 100)}%`,
                  backgroundColor: status === 'paused' ? '#f59e0b' : (status === 'idle' ? '#6b7280' : undefined),
                  transition: 'width 0.5s cubic-bezier(0.4, 0, 0.2, 1)'
                }}
              />
              <div className="progress-bar-text">
                {progress.contracts_scanned} / {progress.contracts_total}
              </div>
            </div>
          )}
          {progress.current_contract_name && status === 'running' && (
            <div className="progress-current">
              🔄 Currently scanning: <code>{progress.current_contract_name}</code>
            </div>
          )}
        </div>
      )}

      <div className="scanner-config">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
          <h2 style={{ margin: 0 }}>Configuration</h2>
          <div className="preset-buttons">
            <button 
              className="btn-preset" 
              onClick={() => applyPreset('quick')}
              disabled={isControlsDisabled}
              title="Quick scan: Last 7 days, Ethereum only, Fast"
            >
              🚀 Quick
            </button>
            <button 
              className="btn-preset" 
              onClick={() => applyPreset('deep')}
              disabled={isControlsDisabled}
              title="Deep scan: Last 30 days, All chains, Full analysis"
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
            <label>
              Contract Addresses <span className="label-hint">(empty = autodetect)</span>
            </label>
            <input
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
            <label>Blockchain Chain</label>
            <select 
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
            <label>
              Days to Scan{' '}
              {config.days === 0 || config.days === '0' ? (
                <span className="label-hint">(no scan, only 0-day reports)</span>
              ) : null}
            </label>
            <input
              type="number"
              name="days"
              value={config.days}
              onChange={handleInputChange}
              min="0"
              max="365"
              disabled={isControlsDisabled}
              className={validationErrors.days ? 'input-error' : ''}
              title="Set to 0 for only 0-day reports without scanning"
            />
            {validationErrors.days && (
              <span className="validation-error">{validationErrors.days}</span>
            )}
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
            <span>Fetch 0-day exploits <small style={{ opacity: 0.6, fontSize: '0.85em' }}>(last 30 days)</small></span>
          </label>
        </div>

        <div className="config-group" style={{ marginTop: '1rem' }}>
          <label>Extract Top Riskiest Sources</label>
          <select 
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
