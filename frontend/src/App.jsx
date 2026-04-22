import React, { useEffect, useState } from 'react'
import Scanner from './components/Scanner'
import './App.css'
import { DEFAULT_RUNTIME_CONFIG, loadRuntimeConfig } from './utils/runtimeConfig'

function App() {
  const [runtimeConfig, setRuntimeConfig] = useState(DEFAULT_RUNTIME_CONFIG)
  const [runtimeState, setRuntimeState] = useState('loading')
  const [runtimeError, setRuntimeError] = useState('')

  useEffect(() => {
    let active = true

    loadRuntimeConfig()
      .then((config) => {
        if (!active) return
        setRuntimeConfig(config)
        setRuntimeState('ready')
      })
      .catch((error) => {
        if (!active) return
        setRuntimeError(error.message)
        setRuntimeState('error')
      })

    return () => {
      active = false
    }
  }, [])

  if (runtimeState === 'loading') {
    return (
      <div className="app-shell">
        <main className="app-main">
          <div className="workspace-shell">
            <div className="status-card">
              <div className="status-eyebrow">Runtime</div>
              <h1 className="status-title">Resolving local desktop configuration</h1>
              <p className="status-description">
                Waiting for the shared SCPF runtime to provide the active backend endpoint.
              </p>
            </div>
          </div>
        </main>
      </div>
    )
  }

  if (runtimeState === 'error') {
    return (
      <div className="app-shell">
        <main className="app-main">
          <div className="workspace-shell">
            <div className="status-card">
              <div className="status-eyebrow">Runtime</div>
              <h1 className="status-title">Desktop boot failed</h1>
              <p className="status-description">{runtimeError}</p>
            </div>
          </div>
        </main>
      </div>
    )
  }

  return (
    <div className="app-shell">
      <div className="app-version">v0.2.0</div>
      <main className="app-main">
        <Scanner apiBaseUrl={runtimeConfig.apiBaseUrl} />
      </main>
    </div>
  )
}

export default App
