import React, { useState, useEffect, useRef, useCallback } from 'react'
import './Console.css'
import {
  API_ENDPOINTS,
  TIMEOUTS,
  CONNECTION_CONFIG,
  CONNECTION_STATUS,
  SCAN_STATUS,
} from '../constants'
import { fetchScanStatus, checkHealth } from '../utils/api'

/**
 * Console component - Displays real-time scan logs with SSE connection
 */
const Console = () => {
  // State
  const [logs, setLogs] = useState([])
  const [connectionState, setConnectionState] = useState(CONNECTION_STATUS.DISCONNECTED)
  const [errorMessage, setErrorMessage] = useState('')
  const [scanStatus, setScanStatus] = useState(SCAN_STATUS.IDLE)
  const [copiedId, setCopiedId] = useState(null)

  // Refs
  const consoleRef = useRef(null)
  const eventSourceRef = useRef(null)
  const reconnectTimeoutRef = useRef(null)
  const statusIntervalRef = useRef(null)
  const isInitializedRef = useRef(false)
  const reconnectAttemptsRef = useRef(0)

  // ===== CALLBACKS =====

  /**
   * Add a log entry with automatic trimming
   */
  const addLog = useCallback((message, type = 'log') => {
    const timestamp = new Date().toLocaleTimeString()
    const id = `${timestamp}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    
    setLogs(prev => {
      const newLog = { id, message, type, timestamp }
      const maxLogs = CONNECTION_CONFIG.MAX_LOG_ENTRIES
      return prev.length >= maxLogs 
        ? [...prev.slice(-(maxLogs - 1)), newLog] 
        : [...prev, newLog]
    })
  }, [])

  /**
   * Clean up SSE connection and timeouts
   */
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

  /**
   * Establish SSE connection to log stream
   */
  const connectToLogs = useCallback(() => {
    cleanupConnection()
    setConnectionState(CONNECTION_STATUS.CONNECTING)
    
    try {
      const eventSource = new EventSource(API_ENDPOINTS.LOGS)
      eventSourceRef.current = eventSource

      const connectionTimeout = setTimeout(() => {
        if (eventSourceRef.current?.readyState !== EventSource.OPEN) {
          setConnectionState(CONNECTION_STATUS.ERROR)
          setErrorMessage('Connection timeout - server may not be responding')
          addLog('⚠️ Connection timeout. Click Reconnect to try again.', 'system')
        }
      }, TIMEOUTS.CONNECTION_TIMEOUT)

      eventSource.onopen = () => {
        clearTimeout(connectionTimeout)
        setConnectionState(CONNECTION_STATUS.CONNECTED)
        setErrorMessage('')
        reconnectAttemptsRef.current = 0
        addLog('🟢 Connected to server', 'system')
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
            `🔴 Connection lost. Reconnecting in ${delay / 1000}s... (attempt ${reconnectAttemptsRef.current}/${CONNECTION_CONFIG.MAX_RECONNECT_ATTEMPTS})`,
            'system'
          )
          
          reconnectTimeoutRef.current = setTimeout(() => {
            performHealthCheck()
          }, delay)
        } else {
          setConnectionState(CONNECTION_STATUS.ERROR)
          setErrorMessage('Max reconnection attempts reached')
          addLog('❌ Max reconnection attempts reached. Click Reconnect to try again.', 'system')
        }
      }
    } catch (error) {
      setConnectionState(CONNECTION_STATUS.ERROR)
      setErrorMessage(error.message)
      addLog('❌ Failed to create connection: ' + error.message, 'system')
    }
  }, [addLog, cleanupConnection])

  /**
   * Check server health before connecting
   */
  const performHealthCheck = useCallback(async () => {
    setConnectionState(CONNECTION_STATUS.CONNECTING)
    addLog('🔍 Checking server health...', 'system')
    
    try {
      await checkHealth()
      addLog('✅ Server is online, connecting...', 'system')
      connectToLogs()
    } catch (error) {
      const isNetworkError = !error.response
      const errorMsg = isNetworkError 
        ? 'Backend server is not running' 
        : `Server error: ${error.response?.status}`
      
      setConnectionState(CONNECTION_STATUS.ERROR)
      setErrorMessage(errorMsg)
      addLog(`❌ ${errorMsg}. Please start it with: cargo run --bin scpf-server`, 'system')
      
      // Auto-retry with exponential backoff
      reconnectAttemptsRef.current += 1
      if (reconnectAttemptsRef.current <= CONNECTION_CONFIG.MAX_RECONNECT_ATTEMPTS) {
        const delay = Math.min(
          TIMEOUTS.RECONNECT_DELAY_BASE * reconnectAttemptsRef.current,
          TIMEOUTS.RECONNECT_DELAY_MAX
        )
        reconnectTimeoutRef.current = setTimeout(performHealthCheck, delay)
      }
    }
  }, [addLog, connectToLogs])

  /**
   * Poll scan status for badge display
   */
  const pollScanStatus = useCallback(async () => {
    try {
      const data = await fetchScanStatus()
      setScanStatus(data.status || SCAN_STATUS.IDLE)
    } catch {
      // Silently fail - connection status is handled elsewhere
    }
  }, [])

  // ===== ACTION HANDLERS =====

  const handleReconnect = useCallback(async () => {
    cleanupConnection()
    reconnectAttemptsRef.current = 0
    setLogs([])
    setErrorMessage('')
    addLog('🔄 Manually reconnecting to server...', 'system')
    
    // Small delay to ensure cleanup is complete
    await new Promise(resolve => setTimeout(resolve, 100))
    performHealthCheck()
  }, [addLog, cleanupConnection, performHealthCheck])

  const clearLogs = useCallback(() => {
    setLogs([])
  }, [])

  const copyLog = useCallback((message, id) => {
    navigator.clipboard.writeText(message)
      .then(() => {
        setCopiedId(id)
        setTimeout(() => setCopiedId(null), TIMEOUTS.COPY_FEEDBACK)
      })
      .catch(err => {
        console.error('Failed to copy:', err)
      })
  }, [])

  const copyAllLogs = useCallback(() => {
    const allLogsText = logs.map(log => `[${log.timestamp}] ${log.message}`).join('\n')
    navigator.clipboard.writeText(allLogsText)
      .then(() => {
        addLog('✅ All logs copied to clipboard', 'system')
      })
      .catch(err => {
        console.error('Failed to copy all logs:', err)
        addLog('❌ Failed to copy logs', 'system')
      })
  }, [logs, addLog])

  // ===== EFFECTS =====

  // Initialize connection and polling
  useEffect(() => {
    if (isInitializedRef.current) return
    isInitializedRef.current = true
    
    performHealthCheck()
    statusIntervalRef.current = setInterval(pollScanStatus, TIMEOUTS.LOG_STATUS_POLL_INTERVAL)
    
    return () => {
      cleanupConnection()
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current)
      }
    }
  }, [performHealthCheck, cleanupConnection, pollScanStatus])

  // Auto-scroll to bottom on new logs
  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight
    }
  }, [logs])

  // ===== COMPUTED VALUES =====

  const getLogClass = (type) => {
    switch (type) {
      case 'error': return 'log-error'
      case 'system': return 'log-system'
      default: return 'log-normal'
    }
  }

  const connectionDisplay = (() => {
    switch (connectionState) {
      case CONNECTION_STATUS.CONNECTED:
        return { icon: '🟢', text: 'Connected', className: 'connected' }
      case CONNECTION_STATUS.CONNECTING:
        return { icon: '🟡', text: 'Connecting...', className: 'connecting' }
      case CONNECTION_STATUS.ERROR:
        return { icon: '🔴', text: 'Error', className: 'disconnected' }
      default:
        return { icon: '🔴', text: 'Disconnected', className: 'disconnected' }
    }
  })()

  const isScanning = scanStatus === SCAN_STATUS.RUNNING || scanStatus === SCAN_STATUS.PAUSED
  const canReconnect = connectionState !== CONNECTION_STATUS.CONNECTED && 
                       connectionState !== CONNECTION_STATUS.CONNECTING

  // ===== RENDER =====

  return (
    <div className="console">
      <header className="console-header">
        <div className="console-title">
          <span className="console-icon" aria-hidden="true">💻</span>
          <h3>Scan Console</h3>
        </div>
        <div className="console-controls">
          <span 
            className={`connection-status ${connectionDisplay.className}`}
            role="status"
            aria-label={`Connection status: ${connectionDisplay.text}`}
          >
            {connectionDisplay.icon} {connectionDisplay.text}
          </span>
          
          {isScanning && (
            <span className="scan-status-badge" role="status">
              {scanStatus === SCAN_STATUS.RUNNING ? '🔄 Scanning...' : '⏸️ Paused'}
            </span>
          )}
          
          {canReconnect && (
            <button 
              onClick={handleReconnect} 
              className="btn-reconnect"
              disabled={connectionState === CONNECTION_STATUS.CONNECTING}
              title={connectionState === CONNECTION_STATUS.CONNECTING ? 'Connecting...' : 'Reconnect to server'}
            >
              🔄 Reconnect
            </button>
          )}
          
          <button 
            onClick={clearLogs} 
            className="btn-clear"
            title="Clear all logs"
            disabled={logs.length === 0 || isScanning}
          >
            🗑️ Clear
          </button>
          
          <button 
            onClick={copyAllLogs} 
            className="btn-copy-all"
            title="Copy all logs to clipboard"
            disabled={logs.length === 0 || isScanning}
          >
            📋 Copy All
          </button>
        </div>
      </header>
      
      {errorMessage && (
        <div className="console-error-banner" role="alert">
          ⚠️ {errorMessage}
        </div>
      )}
      
      <div 
        className="console-output" 
        ref={consoleRef}
        role="log"
        aria-live="polite"
        aria-label="Scan output console"
      >
        {logs.length === 0 ? (
          <div className="console-empty">
            <p>No logs yet. Start a scan to see output here.</p>
          </div>
        ) : (
          logs.map((log) => (
            <div key={log.id} className={`console-line ${getLogClass(log.type)}`}>
              <span className="console-timestamp">[{log.timestamp}]</span>
              <span className="console-message">{log.message}</span>
              <button 
                className="btn-copy-log"
                onClick={() => copyLog(log.message, log.id)}
                title="Copy log message"
                aria-label={`Copy log: ${log.message.substring(0, 50)}`}
              >
                {copiedId === log.id ? '✓' : '📋'}
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default Console
