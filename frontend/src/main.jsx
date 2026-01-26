import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

// Removed StrictMode to prevent double-mounting effects
// which causes duplicate log entries and EventSource connections
ReactDOM.createRoot(document.getElementById('root')).render(<App />)
