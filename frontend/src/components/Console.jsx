import React, { useState, useEffect, useRef, useCallback } from 'react'
import axios from 'axios'
import './Console.css'

const Console = () => {
  const [logs, setLogs] = useState([])
  const [connectionState, setConnectionState] = useState('disconnected') // 'disconnected' | 'connecting' | 'connected' | 'error'
  const [errorMessage, setErrorMessage] = useState('')
  const [scanStatus, setScanStatus] = useState('idle') // 'idle' | 'running' | 'paused' | 'stopped'
  const [isStoppingOrPausing, setIsStoppingOrPausing] = useState(false)
  const consoleRef = useRef(null)
  const eventSourceRef = useRef(null)
  const reconnectTimeoutRef = useRef(null)
  const statusIntervalRef = useRef(null)
  const isInitializedRef = useRef(false)
  const reconnectAttemptsRef = useRef(0)
  const maxReconnectAttempts = 5

  const addLog = useCallback((message, type = 'log') => {
    const timestamp = new Date().toLocaleTimeString()
    const id = `${timestamp}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    setLogs(prev => {
      const newLog = { id, message, type, timestamp }
      return prev.length >= 1000 ? [...prev.slice(-999), newLog] : [...prev, newLog]
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

  const connectToLogs = useCallback(() => {
    cleanupConnection()
    
    setConnectionState('connecting')
    
    try {
      const eventSource = new EventSource('/api/logs')
      eventSourceRef.current = eventSource

      const connectionTimeout = setTimeout(() => {
        if (eventSourceRef.current && eventSourceRef.current.readyState !== EventSource.OPEN) {
          setConnectionState('error')
          setErrorMessage('Connection timeout - server may not be responding')
          addLog('⚠️ Connection timeout. Click Reconnect to try again.', 'system')
        }
      }, 5000)

      eventSource.onopen = () => {
        clearTimeout(connectionTimeout)
        setConnectionState('connected')
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
        setConnectionState('disconnected')
        
        reconnectAttemptsRef.current += 1
        
        if (reconnectAttemptsRef.current <= maxReconnectAttempts) {
          const delay = Math.min(5000 * reconnectAttemptsRef.current, 30000)
          addLog(`🔴 Connection lost. Reconnecting in ${delay / 1000}s... (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})`, 'system')
          
          reconnectTimeoutRef.current = setTimeout(() => {
            checkServerHealth()
          }, delay)
        } else {
          setConnectionState('error')
          setErrorMessage('Max reconnection attempts reached')
          addLog('❌ Max reconnection attempts reached. Click Reconnect to try again.', 'system')
        }
      }
    } catch (error) {
      setConnectionState('error')
      setErrorMessage(error.message)
      addLog('❌ Failed to create connection: ' + error.message, 'system')
    }
  }, [addLog, cleanupConnection])

  const checkServerHealth = useCallback(async () => {
    setConnectionState('connecting')
    addLog('🔍 Checking server health...', 'system')
    
    try {
      await axios.get('/api/health', { timeout: 3000 })
      addLog('✅ Server is online, connecting...', 'system')
      connectToLogs()
    } catch (error) {
      const isNetworkError = !error.response
      const errorMsg = isNetworkError 
        ? 'Backend server is not running' 
        : `Server error: ${error.response?.status}`
      
      setConnectionState('error')
      setErrorMessage(errorMsg)
      addLog(`❌ ${errorMsg}. Please start it with: cargo run --bin scpf-server`, 'system')
      
      // Auto-retry with exponential backoff
      reconnectAttemptsRef.current += 1
      if (reconnectAttemptsRef.current <= maxReconnectAttempts) {
        const delay = Math.min(5000 * reconnectAttemptsRef.current, 30000)
        reconnectTimeoutRef.current = setTimeout(() => {
          checkServerHealth()
        }, delay)
      }
    }
  }, [addLog, connectToLogs])

  const fetchScanStatus = useCallback(async () => {
    try {
      const response = await axios.get('/api/status', { timeout: 2000 })
      setScanStatus(response.data.status || 'idle')
    } catch {
      // Silently fail - connection status is handled elsewhere
    }
  }, [])

  useEffect(() => {
    // Prevent double initialization
    if (isInitializedRef.current) return
    isInitializedRef.current = true
    
    checkServerHealth()
    
    // Poll scan status
    statusIntervalRef.current = setInterval(fetchScanStatus, 1500)
    
    return () => {
      cleanupConnection()
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current)
      }
    }
  }, [checkServerHealth, cleanupConnection, fetchScanStatus])

  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight
    }
  }, [logs])

  const handleReconnect = async () => {
    cleanupConnection()
    reconnectAttemptsRef.current = 0
    setLogs([])
    setErrorMessage('')
    addLog('🔄 Manually reconnecting to server...', 'system')
    
    // Small delay to ensure cleanup is complete
    await new Promise(resolve => setTimeout(resolve, 100))
    checkServerHealth()
  }

  const clearLogs = () => {
    setLogs([])
  }

  const handleStopScan = async () => {
    setIsStoppingOrPausing(true)
    try {
      await axios.post('/api/stop', {}, { timeout: 5000 })
      addLog('🛑 Stop signal sent...', 'system')
      setScanStatus('stopped')
    } catch (err) {
      addLog(`❌ Failed to stop: ${err.message}`, 'error')
    } finally {
      setIsStoppingOrPausing(false)
    }
  }

  const handlePauseScan = async () => {
    setIsStoppingOrPausing(true)
    try {
      await axios.post('/api/pause', {}, { timeout: 5000 })
      addLog('⏸️ Pause signal sent...', 'system')
      setScanStatus('paused')
    } catch (err) {
      addLog(`❌ Failed to pause: ${err.message}`, 'error')
    } finally {
      setIsStoppingOrPausing(false)
    }
  }

  const getLogClass = (type) => {
    if (type === 'error') return 'log-error'
    if (type === 'system') return 'log-system'
    return 'log-normal'
  }

  const getConnectionStatusDisplay = () => {
    switch (connectionState) {
      case 'connected':
        return { icon: '🟢', text: 'Connected', className: 'connected' }
      case 'connecting':
        return { icon: '🟡', text: 'Connecting...', className: 'connecting' }
      case 'error':
        return { icon: '🔴', text: 'Error', className: 'disconnected' }
      default:
        return { icon: '🔴', text: 'Disconnected', className: 'disconnected' }
    }
  }

  const statusDisplay = getConnectionStatusDisplay()

  return (
    <div className="console">
      <div className="console-header">
        <div className="console-title">
          <span className="console-icon">💻</span>
          <h3>Scan Console</h3>
        </div>
        <div className="console-controls">
          <span className={`connection-status ${statusDisplay.className}`}>
            {statusDisplay.icon} {statusDisplay.text}
          </span>
          {/* Control buttons moved to Scanner component - only show status here */}
          {(scanStatus === 'running' || scanStatus === 'paused') && (
            <span className="scan-status-badge">
              {scanStatus === 'running' ? '🔄 Scanning...' : '⏸️ Paused'}
            </span>
          )}
          {connectionState !== 'connected' && (
            <button 
              onClick={handleReconnect} 
              className="btn-reconnect"
              disabled={connectionState === 'connecting'}
              title={connectionState === 'connecting' ? 'Connecting...' : 'Reconnect to server'}
            >
              🔄 Reconnect
            </button>
          )}
          <button onClick={clearLogs} className="btn-clear">
            🗑️ Clear
          </button>
        </div>
      </div>
      
      {errorMessage && (
        <div className="console-error-banner">
          ⚠️ {errorMessage}
        </div>
      )}
      
      <div className="console-output" ref={consoleRef}>
        {logs.length === 0 ? (
          <div className="console-empty">
            <p>No logs yet. Start a scan to see output here.</p>
          </div>
        ) : (
          logs.map((log) => (
            <div key={log.id} className={`console-line ${getLogClass(log.type)}`}>
              <span className="console-timestamp">[{log.timestamp}]</span>
              <span className="console-message">{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default Console
