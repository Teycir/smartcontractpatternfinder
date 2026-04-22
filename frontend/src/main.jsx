import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import DesktopCrashBoundary from './components/DesktopCrashBoundary'
import './index.css'

// Removed StrictMode to prevent double-mounting effects
// which causes duplicate log entries and EventSource connections

const isTauriRuntime =
  typeof window !== 'undefined' &&
  (typeof window.__TAURI__?.core?.invoke === 'function' ||
    typeof window.__TAURI_INTERNALS__?.invoke === 'function')

if (typeof document !== 'undefined') {
  document.documentElement.dataset.runtime = isTauriRuntime ? 'tauri' : 'web'
}

ReactDOM.createRoot(document.getElementById('root')).render(
  <DesktopCrashBoundary>
    <App />
  </DesktopCrashBoundary>
)
