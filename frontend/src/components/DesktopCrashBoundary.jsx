import React, { Component, useEffect, useState } from 'react'
import './DesktopCrashBoundary.css'

function normalizeErrorDetails(reason) {
  if (!reason) {
    return ''
  }

  if (reason instanceof Error) {
    return reason.stack || reason.message || 'Unknown desktop error'
  }

  if (typeof reason === 'string') {
    return reason.trim()
  }

  if (typeof reason === 'object') {
    if (typeof reason.message === 'string' && reason.message.trim()) {
      return reason.message.trim()
    }

    try {
      return JSON.stringify(reason)
    } catch {
      return String(reason)
    }
  }

  return String(reason)
}

function CrashScreen({ title, description, details }) {
  return (
    <div className="app-shell crash-shell">
      <main className="app-main">
        <section className="crash-card" role="alert" aria-live="assertive">
          <span className="crash-eyebrow">Desktop Recovery</span>
          <h1>{title}</h1>
          <p>{description}</p>

          {details && <pre className="crash-details">{details}</pre>}

          <div className="crash-actions">
            <button type="button" className="crash-button" onClick={() => window.location.reload()}>
              Reload Desktop UI
            </button>
          </div>
        </section>
      </main>
    </div>
  )
}

class ReactCrashBoundary extends Component {
  constructor(props) {
    super(props)
    this.state = {
      details: '',
      hasError: false,
    }
  }

  static getDerivedStateFromError(error) {
    return {
      details: normalizeErrorDetails(error),
      hasError: true,
    }
  }

  componentDidCatch(error, errorInfo) {
    const extraDetails = errorInfo?.componentStack?.trim()
    const details = [normalizeErrorDetails(error), extraDetails].filter(Boolean).join('\n\n')

    console.error('Desktop UI crashed.', error, errorInfo)
    this.setState({
      details,
      hasError: true,
    })
  }

  render() {
    if (this.state.hasError) {
      return (
        <CrashScreen
          title="Desktop UI hit an unexpected error"
          description="The window stayed open, but the React workspace failed. Reloading the desktop UI usually restores the session."
          details={this.state.details}
        />
      )
    }

    return this.props.children
  }
}

function DesktopCrashBoundary({ children }) {
  const [asyncErrorDetails, setAsyncErrorDetails] = useState('')

  useEffect(() => {
    const handleWindowError = (event) => {
      const details = normalizeErrorDetails(event.error || event.message)
      if (!details) {
        return
      }

      console.error('Unhandled desktop error.', event.error || event.message)
      setAsyncErrorDetails((currentDetails) => currentDetails || details)
    }

    const handleUnhandledRejection = (event) => {
      const details = normalizeErrorDetails(event.reason)
      if (!details) {
        return
      }

      console.error('Unhandled desktop rejection.', event.reason)
      setAsyncErrorDetails((currentDetails) => currentDetails || details)
    }

    window.addEventListener('error', handleWindowError)
    window.addEventListener('unhandledrejection', handleUnhandledRejection)

    return () => {
      window.removeEventListener('error', handleWindowError)
      window.removeEventListener('unhandledrejection', handleUnhandledRejection)
    }
  }, [])

  if (asyncErrorDetails) {
    return (
      <CrashScreen
        title="Desktop runtime hit an unexpected error"
        description="A browser-level error interrupted the desktop workspace. Reloading should recover the local UI without restarting the whole app."
        details={asyncErrorDetails}
      />
    )
  }

  return <ReactCrashBoundary>{children}</ReactCrashBoundary>
}

export default DesktopCrashBoundary
