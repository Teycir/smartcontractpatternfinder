import React, { useState, useEffect, useRef, useCallback } from 'react'
import './Console.css'
import {
  API_ENDPOINTS,
  TIMEOUTS,
  CONNECTION_CONFIG,
  CONNECTION_STATUS,
  SCAN_STATUS,
} from '../constants'
import { fetchScanStatus, checkHealth, exportLogs, getApiUrl } from '../utils/api'

const Console = () => {
  const [logs, setLogs] = useState([])
  const [connectionState, setConnectionState] = useState(CONNECTION_STATUS.DISCONNECTED)
  const [errorMessage, setErrorMessage] = useState('')
  const [scanStatus, setScanStatus] = useState(SCAN_STATUS.IDLE)
  const [copiedId, setCopiedId] = useState(null)
  const [isExporting, setIsExporting] = useState(false)

  const consoleRef = useRef(null)
  const eventSourceRef = useRef(null)
  const reconnectTimeoutRef = useRef(null)
  const statusIntervalRef = useRef(null)
  const reconnectAttemptsRef = useRef(0)

  const addLog = useCallback((message, type = 'log') => {
    const timestamp = new Date().toLocaleTimeString()
    const id = `${timestamp}-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`

    setLogs((prev) => {
      const newEntry = { id, message, type, timestamp }
      const maxLogs = CONNECTION_CONFIG.MAX_LOG_ENTRIES

      return prev.length >= maxLogs
        ? [...prev.slice(-(maxLogs - 1)), newEntry]
        : [...prev, newEntry]
    })
  }, [])

  const cleanupConnection = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close()
      eventSourceRef.current = null
    }

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }
  }, [])

  const connectToLogs = useCallback(async () => {
    cleanupConnection()
    setConnectionState(CONNECTION_STATUS.CONNECTING)

    try {
      const logsUrl = await getApiUrl(API_ENDPOINTS.LOGS)
      const eventSource = new EventSource(logsUrl)
      eventSourceRef.current = eventSource

      const connectionTimeout = setTimeout(() => {
        if (eventSourceRef.current?.readyState !== EventSource.OPEN) {
          setConnectionState(CONNECTION_STATUS.ERROR)
          setErrorMessage('Connection timeout. The server may still be booting.')
          addLog('Connection timeout. Use reconnect once the local server is healthy.', 'system')
        }
      }, TIMEOUTS.CONNECTION_TIMEOUT)

      eventSource.onopen = () => {
        clearTimeout(connectionTimeout)
        setConnectionState(CONNECTION_STATUS.CONNECTED)
        setErrorMessage('')
        reconnectAttemptsRef.current = 0
        addLog('Live log stream connected.', 'system')
      }

      eventSource.onmessage = (event) => {
        if (event.data && event.data !== 'keep-alive' && event.data !== 'Connected') {
          addLog(event.data, 'log')
        }
      }

      eventSource.onerror = () => {
        clearTimeout(connectionTimeout)
        cleanupConnection()
        setConnectionState(CONNECTION_STATUS.DISCONNECTED)

        reconnectAttemptsRef.current += 1

        if (reconnectAttemptsRef.current <= CONNECTION_CONFIG.MAX_RECONNECT_ATTEMPTS) {
          const delay = Math.min(
            TIMEOUTS.RECONNECT_DELAY_BASE * reconnectAttemptsRef.current,
            TIMEOUTS.RECONNECT_DELAY_MAX
          )
          addLog(
            `Log stream disconnected. Reconnecting in ${delay / 1000}s (${reconnectAttemptsRef.current}/${CONNECTION_CONFIG.MAX_RECONNECT_ATTEMPTS}).`,
            'system'
          )

          reconnectTimeoutRef.current = setTimeout(() => {
            void performHealthCheck()
          }, delay)
        } else {
          setConnectionState(CONNECTION_STATUS.ERROR)
          setErrorMessage('Maximum reconnection attempts reached.')
          addLog('Maximum reconnection attempts reached. Manual reconnect is required.', 'error')
        }
      }
    } catch (error) {
      setConnectionState(CONNECTION_STATUS.ERROR)
      setErrorMessage(error.message)
      addLog(`Failed to create the log stream: ${error.message}`, 'error')
    }
  }, [addLog, cleanupConnection])

  const performHealthCheck = useCallback(async () => {
    setConnectionState(CONNECTION_STATUS.CONNECTING)

    try {
      await checkHealth()
      setErrorMessage('')
      await connectToLogs()
    } catch (error) {
      const networkError = !error.response
      const nextError = networkError
        ? 'Backend server is not responding yet.'
        : `Backend returned ${error.response?.status}.`

      setConnectionState(CONNECTION_STATUS.ERROR)
      setErrorMessage(nextError)

      reconnectAttemptsRef.current += 1
      if (reconnectAttemptsRef.current <= CONNECTION_CONFIG.MAX_RECONNECT_ATTEMPTS) {
        const delay = Math.min(
          TIMEOUTS.RECONNECT_DELAY_BASE * reconnectAttemptsRef.current,
          TIMEOUTS.RECONNECT_DELAY_MAX
        )
        reconnectTimeoutRef.current = setTimeout(() => {
          void performHealthCheck()
        }, delay)
      }
    }
  }, [connectToLogs])

  const pollScanStatus = useCallback(async () => {
    try {
      const data = await fetchScanStatus()
      setScanStatus(data.status || SCAN_STATUS.IDLE)
    } catch {
      // Connection failures are surfaced in the dedicated console connection state.
    }
  }, [])

  const handleReconnect = useCallback(async () => {
    cleanupConnection()
    reconnectAttemptsRef.current = 0
    setLogs([])
    setErrorMessage('')
    addLog('Manual reconnect requested. Re-checking local server health.', 'system')
    await new Promise((resolve) => setTimeout(resolve, 120))
    await performHealthCheck()
  }, [addLog, cleanupConnection, performHealthCheck])

  const clearLogs = useCallback(() => {
    setLogs([])
  }, [])

  const copyText = useCallback(
    async (text, successMessage, failureMessage) => {
      if (!navigator.clipboard?.writeText) {
        addLog(failureMessage, 'error')
        return false
      }

      try {
        await navigator.clipboard.writeText(text)
        if (successMessage) {
          addLog(successMessage, 'system')
        }
        return true
      } catch (error) {
        console.error('Copy failed:', error)
        addLog(failureMessage, 'error')
        return false
      }
    },
    [addLog]
  )

  const copyLog = useCallback(
    async (message, id) => {
      const copied = await copyText(message, '', 'Failed to copy the selected log line.')
      if (copied) {
        setCopiedId(id)
        setTimeout(() => setCopiedId(null), TIMEOUTS.COPY_FEEDBACK)
      }
    },
    [copyText]
  )

  const copyAllLogs = useCallback(async () => {
    const allLogsText = logs.map((log) => `[${log.timestamp}] ${log.message}`).join('\n')
    await copyText(allLogsText, 'Full console log copied to clipboard.', 'Failed to copy console log.')
  }, [copyText, logs])

  const handleExportLogs = useCallback(async () => {
    if (logs.length === 0 || isExporting) return

    setIsExporting(true)
    try {
      const allLogsText = logs.map((log) => `[${log.timestamp}] ${log.message}`).join('\n')
      const response = await exportLogs(allLogsText)
      addLog(`Console log exported to ${response.path}`, 'system')
    } catch (error) {
      addLog(`Failed to export console log: ${error.response?.data?.error || error.message}`, 'error')
    } finally {
      setIsExporting(false)
    }
  }, [addLog, isExporting, logs])

  useEffect(() => {
    void performHealthCheck()
    statusIntervalRef.current = setInterval(pollScanStatus, TIMEOUTS.LOG_STATUS_POLL_INTERVAL)

    return () => {
      cleanupConnection()
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current)
      }
    }
  }, [cleanupConnection, performHealthCheck, pollScanStatus])

  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight
    }
  }, [logs])

  const connectionDisplay = (() => {
    switch (connectionState) {
      case CONNECTION_STATUS.CONNECTED:
        return { text: 'Connected', className: 'connected' }
      case CONNECTION_STATUS.CONNECTING:
        return { text: 'Connecting', className: 'connecting' }
      case CONNECTION_STATUS.ERROR:
        return { text: 'Attention needed', className: 'error' }
      default:
        return { text: 'Disconnected', className: 'disconnected' }
    }
  })()

  const isScanning = scanStatus === SCAN_STATUS.RUNNING || scanStatus === SCAN_STATUS.PAUSED
  const canReconnect =
    connectionState !== CONNECTION_STATUS.CONNECTED &&
    connectionState !== CONNECTION_STATUS.CONNECTING

  const getLogClass = (type) => {
    switch (type) {
      case 'error':
        return 'log-error'
      case 'system':
        return 'log-system'
      default:
        return 'log-normal'
    }
  }

  return (
    <div className="console">
      <header className="console-header">
        <div className="console-title-block">
          <div className="console-title-row">
            <h4>Session Stream</h4>
            <span className={`connection-pill ${connectionDisplay.className}`}>
              {connectionDisplay.text}
            </span>
          </div>
          <p>
            {isScanning
              ? 'Live output is being streamed directly from the current scan.'
              : 'The console stays attached and ready for the next local run.'}
          </p>
        </div>

        <div className="console-controls">
          <span className="console-count">{logs.length} entries</span>

          {canReconnect && (
            <button type="button" className="console-button" onClick={handleReconnect}>
              Reconnect
            </button>
          )}

          <button
            type="button"
            className="console-button"
            onClick={copyAllLogs}
            disabled={logs.length === 0}
          >
            Copy all
          </button>

          <button
            type="button"
            className="console-button"
            onClick={handleExportLogs}
            disabled={logs.length === 0 || isExporting}
          >
            {isExporting ? 'Exporting…' : 'Export'}
          </button>

          <button
            type="button"
            className="console-button console-button-subtle"
            onClick={clearLogs}
            disabled={logs.length === 0}
          >
            Clear
          </button>
        </div>
      </header>

      {errorMessage && (
        <div className="console-error-banner" role="alert">
          {errorMessage}
        </div>
      )}

      <div className="console-output" ref={consoleRef} role="log" aria-live="polite">
        {logs.length === 0 ? (
          <div className="console-empty">
            <p>No log lines yet. Start a scan to populate the live session stream.</p>
          </div>
        ) : (
          logs.map((log) => (
            <div key={log.id} className={`console-line ${getLogClass(log.type)}`}>
              <span className="console-timestamp">[{log.timestamp}]</span>
              <span className="console-message">{log.message}</span>
              <button
                type="button"
                className="console-copy-button"
                onClick={() => copyLog(log.message, log.id)}
                aria-label={`Copy log line: ${log.message.substring(0, 60)}`}
              >
                {copiedId === log.id ? 'Copied' : 'Copy'}
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default Console
