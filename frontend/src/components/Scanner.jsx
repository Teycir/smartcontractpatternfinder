import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import Console from './Console'
import TemplateSelector from './TemplateSelector'
import './Scanner.css'
import {
  TIMEOUTS,
  INITIAL_PROGRESS,
  INITIAL_CONFIG,
  SCAN_PRESETS,
  SCAN_STATUS,
} from '../constants'
import {
  validatePages,
  validateConcurrency,
  buildScanPayload,
  parseAddresses,
} from '../utils/validation'
import {
  fetchScanStatus,
  startScan,
  pauseScan,
  resumeScan,
  stopScan,
  getErrorMessage,
} from '../utils/api'

const SECTION_LINKS = [
  { id: 'overview', label: 'Overview' },
  { id: 'scope', label: 'Scan Scope' },
  { id: 'strategy', label: 'Execution' },
  { id: 'templates', label: 'Templates' },
  { id: 'console', label: 'Console' },
]

const PRESET_DETAILS = {
  quick: {
    label: 'Quick Sweep',
    title: 'Fast signal pass',
    description: 'Pull a focused sample and keep the scanner responsive for short local runs.',
  },
  deep: {
    label: 'Deep Audit',
    title: 'Broader coverage',
    description: 'Expand the fetch window and extraction set when you want fuller local visibility.',
  },
  zeroday: {
    label: '0-Day Watch',
    title: 'Exploit intel only',
    description: 'Skip contract fetching and generate a recent exploit watchlist.',
  },
}

const CHAIN_OPTIONS = [
  { value: 'all', label: 'All supported chains' },
  { value: 'ethereum', label: 'Ethereum' },
  { value: 'polygon', label: 'Polygon' },
  { value: 'arbitrum', label: 'Arbitrum' },
]

const CONTRACT_TYPE_OPTIONS = [
  { value: '', label: 'Any contract type' },
  { value: 'erc20', label: 'ERC-20' },
  { value: 'erc721', label: 'ERC-721' },
  { value: 'erc1155', label: 'ERC-1155' },
  { value: 'proxy', label: 'Proxy' },
  { value: 'defi', label: 'DeFi' },
]

const EXTRACT_OPTIONS = [
  { value: '10', label: 'Top 10 riskiest sources' },
  { value: '25', label: 'Top 25 riskiest sources' },
  { value: '50', label: 'Top 50 riskiest sources' },
  { value: '100', label: 'Top 100 riskiest sources' },
  { value: '200', label: 'Top 200 riskiest sources' },
  { value: '0', label: 'Skip source extraction' },
]

function stripTemplateExtension(templateName) {
  return templateName.replace(/\.(yaml|yml)$/i, '')
}

const Scanner = ({ apiBaseUrl }) => {
  const [status, setStatus] = useState(SCAN_STATUS.IDLE)
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [serverOnline, setServerOnline] = useState(false)
  const [validationErrors, setValidationErrors] = useState({})
  const [progress, setProgress] = useState(INITIAL_PROGRESS)
  const [config, setConfig] = useState(INITIAL_CONFIG)
  const [selectedTemplates, setSelectedTemplates] = useState([])

  const statusIntervalRef = useRef(null)
  const errorTimeoutRef = useRef(null)

  const showError = useCallback((message) => {
    setError(message)

    if (errorTimeoutRef.current) {
      clearTimeout(errorTimeoutRef.current)
    }

    errorTimeoutRef.current = setTimeout(() => {
      setError('')
    }, TIMEOUTS.ERROR_DISPLAY)
  }, [])

  const clearValidationError = useCallback((field) => {
    setValidationErrors((prev) => {
      const { [field]: removed, ...rest } = prev
      return rest
    })
  }, [])

  const pollStatus = useCallback(async () => {
    try {
      const data = await fetchScanStatus()
      const currentStatus = data.status || SCAN_STATUS.IDLE

      setServerOnline(true)
      setStatus(currentStatus)

      if (data.progress) {
        setProgress(data.progress)
      }

      if (data.config && currentStatus !== SCAN_STATUS.IDLE) {
        const cfg = data.config
        setConfig((prev) => ({
          ...prev,
          addresses: cfg.addresses?.join(',\n') || prev.addresses,
          chain: cfg.chain === 'ethereum,polygon,arbitrum' ? 'all' : cfg.chain || prev.chain,
          pages: cfg.pages ?? prev.pages,
          concurrency: cfg.concurrency ?? prev.concurrency,
          tags: cfg.tags || '',
          contract_type: cfg.contract_type || '',
          extract_sources:
            cfg.extract_sources != null ? cfg.extract_sources.toString() : prev.extract_sources,
          fetch_zero_day:
            cfg.fetch_zero_day != null ? Boolean(cfg.fetch_zero_day) : prev.fetch_zero_day,
        }))
      }
    } catch (err) {
      setServerOnline(false)
      if (err.code !== 'ECONNABORTED') {
        console.debug('Status fetch failed:', err.message)
      }
    }
  }, [])

  const handleStart = useCallback(async () => {
    if (isLoading) return

    setIsLoading(true)
    setError('')
    setValidationErrors({})

    try {
      const pagesValidation = validatePages(config.pages)
      if (!pagesValidation.isValid) {
        throw new Error(pagesValidation.error)
      }

      const concurrencyValidation = validateConcurrency(config.concurrency)
      if (!concurrencyValidation.isValid) {
        throw new Error(concurrencyValidation.error)
      }

      const payload = { ...buildScanPayload(config), templates: selectedTemplates }
      await startScan(payload)
      setStatus(SCAN_STATUS.RUNNING)
    } catch (err) {
      showError(getErrorMessage(err, 'Failed to start scan'))
    } finally {
      setIsLoading(false)
    }
  }, [config, isLoading, selectedTemplates, showError])

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

  const handleInputChange = useCallback(
    (event) => {
      const { name, value, type, checked } = event.target
      const nextValue = type === 'checkbox' ? checked : value

      setConfig((prev) => ({ ...prev, [name]: nextValue }))

      if (name === 'pages') {
        const validation = validatePages(value)
        if (!validation.isValid) {
          setValidationErrors((prev) => ({ ...prev, pages: validation.error }))
        } else {
          clearValidationError('pages')
        }
      }

      if (name === 'concurrency') {
        const validation = validateConcurrency(value)
        if (!validation.isValid) {
          setValidationErrors((prev) => ({ ...prev, concurrency: validation.error }))
        } else {
          clearValidationError('concurrency')
        }
      }
    },
    [clearValidationError]
  )

  const applyPreset = useCallback((presetName) => {
    const preset = SCAN_PRESETS[presetName]
    if (!preset) return

    setConfig((prev) => ({ ...prev, ...preset }))
    setValidationErrors({})
  }, [])

  const scrollToSection = useCallback((sectionId) => {
    const target = document.getElementById(sectionId)
    if (target) {
      target.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }, [])

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

  const parsedAddresses = useMemo(() => parseAddresses(config.addresses), [config.addresses])

  const isZeroDayMode = config.pages === 0 || config.pages === '0'
  const isScanActive = status === SCAN_STATUS.RUNNING || status === SCAN_STATUS.PAUSED
  const isConfigLocked = isScanActive || isLoading

  const progressPercent = useMemo(() => {
    if (!progress.contracts_total) return 0
    return Math.min(100, (progress.contracts_scanned / progress.contracts_total) * 100)
  }, [progress.contracts_scanned, progress.contracts_total])

  const etaDisplay = useMemo(() => {
    const eta = progress.eta_seconds
    if (!eta || Number.isNaN(eta) || eta <= 0) return 'Calculating'

    const minutes = Math.floor(eta / 60)
    const seconds = Math.floor(eta % 60)
    return `${minutes}m ${seconds}s`
  }, [progress.eta_seconds])

  const statusMeta = useMemo(() => {
    if (!serverOnline) {
      return {
        label: 'Backend unavailable',
        description: `Waiting for the local Axum service on ${apiBaseUrl}.`,
        tone: 'offline',
      }
    }

    switch (status) {
      case SCAN_STATUS.RUNNING:
        return {
          label: 'Scan in progress',
          description: 'The scanner is actively consuming contracts and streaming logs.',
          tone: 'running',
        }
      case SCAN_STATUS.PAUSED:
        return {
          label: 'Scan paused',
          description: 'Execution is suspended and can resume without losing the current queue.',
          tone: 'paused',
        }
      default:
        return {
          label: 'Ready to launch',
          description: 'Local runtime is healthy and waiting for the next configured scan.',
          tone: 'ready',
        }
    }
  }, [apiBaseUrl, serverOnline, status])

  const summaryCards = useMemo(
    () => [
      {
        label: 'Backend',
        value: serverOnline ? 'Connected' : 'Waiting',
        hint: serverOnline ? 'Local service is responding.' : 'Desktop shell is reconnecting.',
      },
      {
        label: 'Template coverage',
        value: `${selectedTemplates.length}`,
        hint:
          selectedTemplates.length > 0
            ? 'Templates saved in local browser storage.'
            : 'Selection loads as soon as templates sync.',
      },
      {
        label: 'Target scope',
        value:
          parsedAddresses.length > 0
            ? `${parsedAddresses.length} queued`
            : config.chain === 'all'
              ? 'All chains'
              : config.chain,
        hint:
          parsedAddresses.length > 0
            ? 'Manual contract addresses override automatic discovery.'
            : 'Recent-chain discovery stays enabled.',
      },
      {
        label: '0-day watch',
        value: config.fetch_zero_day ? 'Enabled' : 'Disabled',
        hint: config.fetch_zero_day ? 'Recent exploit reports join the scan.' : 'Live scan only.',
      },
    ],
    [config.chain, config.fetch_zero_day, parsedAddresses.length, selectedTemplates.length, serverOnline]
  )

  const progressCards = useMemo(
    () => [
      {
        label: 'Contracts scanned',
        value: progress.contracts_total
          ? `${progress.contracts_scanned}/${progress.contracts_total}`
          : `${progress.contracts_scanned}`,
      },
      {
        label: 'Sources extracted',
        value: `${progress.contracts_extracted || 0}`,
      },
      {
        label: 'Critical findings',
        value: `${progress.critical_findings || 0}`,
      },
      {
        label: 'ETA / rate',
        value: `${etaDisplay}${progress.rate ? ` • ${progress.rate.toFixed(1)}/s` : ''}`,
      },
    ],
    [
      etaDisplay,
      progress.contracts_extracted,
      progress.contracts_scanned,
      progress.contracts_total,
      progress.critical_findings,
      progress.rate,
    ]
  )

  const templatePreview = useMemo(() => {
    const preview = selectedTemplates.slice(0, 6).map(stripTemplateExtension)
    if (selectedTemplates.length > 6) {
      preview.push(`+${selectedTemplates.length - 6} more`)
    }
    return preview
  }, [selectedTemplates])

  const currentContractLabel = progress.current_contract_name || progress.current_contract

  return (
    <div className="workspace-shell">
      {error && (
        <div
          className="error-toast"
          onClick={() => setError('')}
          role="alert"
          aria-live="assertive"
        >
          <div className="error-toast-content">
            <span className="error-toast-indicator" />
            <span>{error}</span>
          </div>
          <div className="error-toast-hint">Dismiss</div>
        </div>
      )}

      <aside className="workspace-sidebar">
        <div className="workspace-brand">
          <div className="workspace-brand-mark">SCPF</div>
          <div>
            <p className="workspace-kicker">Desktop Scan Workspace</p>
            <h1>Smart Contract Pattern Finder</h1>
            <p className="workspace-copy">
              Local-first contract auditing with a calmer, more premium operator shell.
            </p>
          </div>
        </div>

        <div className={`sidebar-status sidebar-status-${statusMeta.tone}`}>
          <span className="sidebar-status-dot" />
          <div>
            <strong>{statusMeta.label}</strong>
            <p>{statusMeta.description}</p>
          </div>
        </div>

        <nav className="workspace-nav" aria-label="Workspace sections">
          {SECTION_LINKS.map((section) => (
            <button key={section.id} type="button" onClick={() => scrollToSection(section.id)}>
              {section.label}
            </button>
          ))}
        </nav>

        <div className="sidebar-footnote">
          Same workflow as the local web app: configure, launch, monitor, pause, resume, stop.
        </div>
      </aside>

      <div className="workspace-main">
        <section id="overview" className="panel hero-panel">
          <div className="hero-copy">
            <span className="section-eyebrow">Operator View</span>
            <h2>Keep the full local scan flow, but give it a stronger desktop presence.</h2>
            <p>
              Configure scan scope, control the runtime, and watch the log stream without leaving a
              single local workspace.
            </p>

            <div className="hero-actions">
              <button
                type="button"
                onClick={handleStart}
                disabled={isScanActive || isLoading || !serverOnline}
                className="action-button action-primary"
                title={!serverOnline ? 'Backend server is offline' : 'Start scan'}
              >
                {isLoading && !isScanActive ? 'Starting…' : 'Launch Scan'}
              </button>
              <button
                type="button"
                onClick={status === SCAN_STATUS.PAUSED ? handleResume : handlePause}
                disabled={!isScanActive || isLoading}
                className="action-button action-secondary"
              >
                {status === SCAN_STATUS.PAUSED ? 'Resume Run' : 'Pause Run'}
              </button>
              <button
                type="button"
                onClick={handleStop}
                disabled={!isScanActive || isLoading}
                className="action-button action-danger"
              >
                Stop Run
              </button>
            </div>

            {!serverOnline && (
              <div className="status-banner">
                The desktop shell is waiting for the local server to come online. Controls unlock as
                soon as the health check succeeds.
              </div>
            )}
          </div>

          <div className="hero-aside">
            <div className={`status-chip status-chip-${statusMeta.tone}`}>{statusMeta.label}</div>

            <div className="metrics-grid">
              {summaryCards.map((card) => (
                <article key={card.label} className="metric-card">
                  <span className="metric-label">{card.label}</span>
                  <strong className="metric-value">{card.value}</strong>
                  <p className="metric-hint">{card.hint}</p>
                </article>
              ))}
            </div>

            <div className="progress-shell" data-running={status === SCAN_STATUS.RUNNING}>
              <div className="progress-topline">
                <span className="progress-title">Live progress</span>
                <span className="progress-value">
                  {progress.contracts_total ? `${progressPercent.toFixed(1)}%` : statusMeta.label}
                </span>
              </div>

              <div className="progress-track" role="progressbar" aria-valuenow={progressPercent} aria-valuemin="0" aria-valuemax="100">
                <div
                  className="progress-fill"
                  style={{ width: progress.contracts_total ? `${progressPercent}%` : '18%' }}
                />
              </div>

              <div className="progress-caption">
                {currentContractLabel ? (
                  <>
                    Scanning <code>{currentContractLabel}</code>
                  </>
                ) : (
                  'Progress details appear here as soon as the scanner publishes checkpoints.'
                )}
              </div>

              <div className="progress-meta-grid">
                {progressCards.map((item) => (
                  <div key={item.label} className="progress-meta-card">
                    <span>{item.label}</span>
                    <strong>{item.value}</strong>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>

        <div className="workspace-grid">
          <section id="scope" className="panel section-panel">
            <div className="panel-header">
              <span className="section-eyebrow">Scan Scope</span>
              <h3>Define what the run should inspect</h3>
              <p>
                Leave addresses empty for automatic discovery, or pin exact contracts for a focused
                review.
              </p>
            </div>

            <div className="form-grid">
              <label className="field-group field-group-wide">
                <span className="field-label">
                  Contract addresses
                  <small>Comma or newline separated</small>
                </span>
                <textarea
                  name="addresses"
                  value={config.addresses}
                  onChange={handleInputChange}
                  rows="5"
                  placeholder={'0x1234...\n0x5678...'}
                  disabled={isConfigLocked}
                  className="field-control field-control-area"
                />
                <span className="field-note">
                  {parsedAddresses.length > 0
                    ? `${parsedAddresses.length} manual target${parsedAddresses.length === 1 ? '' : 's'} queued.`
                    : 'No manual addresses entered. Recent contracts will be discovered automatically.'}
                </span>
              </label>

              <label className="field-group">
                <span className="field-label">
                  Contract type
                  <small>Optional structural filter</small>
                </span>
                <select
                  name="contract_type"
                  value={config.contract_type}
                  onChange={handleInputChange}
                  disabled={isConfigLocked}
                  className="field-control"
                >
                  {CONTRACT_TYPE_OPTIONS.map((option) => (
                    <option key={option.value || 'all'} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="field-group">
                <span className="field-label">
                  Vulnerability tags
                  <small>Optional template filter</small>
                </span>
                <input
                  type="text"
                  name="tags"
                  value={config.tags}
                  onChange={handleInputChange}
                  placeholder="reentrancy,access-control"
                  disabled={isConfigLocked}
                  className="field-control"
                />
                <span className="field-note">
                  Narrow detection to specific vulnerability families when you already know the hunt.
                </span>
              </label>
            </div>
          </section>

          <section id="strategy" className="panel section-panel">
            <div className="panel-header">
              <span className="section-eyebrow">Execution Profile</span>
              <h3>Shape how aggressively the local engine runs</h3>
              <p>Presets are shortcuts. You can still fine-tune every control below them.</p>
            </div>

            <div className="preset-grid">
              {Object.entries(PRESET_DETAILS).map(([presetName, presetMeta]) => (
                <button
                  key={presetName}
                  type="button"
                  className="preset-card"
                  onClick={() => applyPreset(presetName)}
                  disabled={isConfigLocked}
                >
                  <span className="preset-label">{presetMeta.label}</span>
                  <strong>{presetMeta.title}</strong>
                  <p>{presetMeta.description}</p>
                </button>
              ))}
            </div>

            <div className="form-grid">
              <label className="field-group">
                <span className="field-label">
                  Chain selection
                  <small>Discovery source</small>
                </span>
                <select
                  name="chain"
                  value={config.chain}
                  onChange={handleInputChange}
                  disabled={isConfigLocked}
                  className="field-control"
                >
                  {CHAIN_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>

              <label className="field-group">
                <span className="field-label">
                  Pages to fetch
                  <small>{isZeroDayMode ? '0 keeps the run in exploit-intel mode only.' : '100 contracts per page.'}</small>
                </span>
                <input
                  type="number"
                  name="pages"
                  value={config.pages}
                  onChange={handleInputChange}
                  min="0"
                  max="100"
                  disabled={isConfigLocked}
                  className={`field-control ${validationErrors.pages ? 'field-control-error' : ''}`}
                />
                {validationErrors.pages && (
                  <span className="field-error" role="alert">
                    {validationErrors.pages}
                  </span>
                )}
              </label>

              <label className="field-group">
                <span className="field-label">
                  Concurrency
                  <small>Parallel workers</small>
                </span>
                <input
                  type="number"
                  name="concurrency"
                  value={config.concurrency}
                  onChange={handleInputChange}
                  min="1"
                  max="20"
                  disabled={isConfigLocked}
                  className={`field-control ${validationErrors.concurrency ? 'field-control-error' : ''}`}
                />
                {validationErrors.concurrency && (
                  <span className="field-error" role="alert">
                    {validationErrors.concurrency}
                  </span>
                )}
              </label>

              <label className="field-group">
                <span className="field-label">
                  Source extraction
                  <small>Post-scan source capture</small>
                </span>
                <select
                  name="extract_sources"
                  value={config.extract_sources}
                  onChange={handleInputChange}
                  disabled={isConfigLocked}
                  className="field-control"
                >
                  {EXTRACT_OPTIONS.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            <label className="toggle-card">
              <div>
                <span>Include 0-day exploit pull</span>
                <p>Fetch recent disclosures from the last 30 days and add them to the local report set.</p>
              </div>
              <span className="toggle-switch">
                <input
                  type="checkbox"
                  name="fetch_zero_day"
                  checked={config.fetch_zero_day}
                  onChange={handleInputChange}
                  disabled={isConfigLocked}
                />
                <span aria-hidden="true" />
              </span>
            </label>
          </section>
        </div>

        <section id="templates" className="panel section-panel">
          <div className="panel-header">
            <span className="section-eyebrow">Template Matrix</span>
            <h3>Control the detection surface without leaving the workspace</h3>
            <p>
              Template selection stays local and persists between sessions, so desktop and web flows
              behave the same way.
            </p>
          </div>

          <div className="template-preview-row">
            {templatePreview.length > 0 ? (
              templatePreview.map((template) => (
                <span key={template} className="template-pill">
                  {template}
                </span>
              ))
            ) : (
              <span className="template-pill template-pill-muted">Waiting for templates to sync…</span>
            )}
          </div>

          <TemplateSelector
            selectedTemplates={selectedTemplates}
            onChange={setSelectedTemplates}
            disabled={isConfigLocked}
          />
        </section>

        <section id="console" className="panel section-panel">
          <div className="panel-header">
            <span className="section-eyebrow">Live Console</span>
            <h3>Follow the run in real time</h3>
            <p>Reconnect, clear, copy, or export the session log without leaving the workspace.</p>
          </div>
          <Console />
        </section>
      </div>
    </div>
  )
}

export default Scanner
