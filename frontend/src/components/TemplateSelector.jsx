import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react'
import './TemplateSelector.css'
import { fetchTemplates } from '../utils/api'

const STORAGE_KEY = 'selectedTemplates'

const TemplateSelector = ({ selectedTemplates, onChange, disabled }) => {
  const [templates, setTemplates] = useState([])
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(true)
  const [query, setQuery] = useState('')
  const [error, setError] = useState('')

  const dropdownRef = useRef(null)
  const initializedRef = useRef(false)

  useEffect(() => {
    if (initializedRef.current) return
    initializedRef.current = true

    setLoading(true)
    fetchTemplates()
      .then((data) => {
        const templateList = data.templates || []
        setTemplates(templateList)
        setError('')

        const saved = localStorage.getItem(STORAGE_KEY)
        if (saved) {
          try {
            const parsed = JSON.parse(saved)
            onChange(parsed.filter((template) => templateList.includes(template)))
          } catch {
            onChange(templateList)
          }
        } else {
          onChange(templateList)
        }
      })
      .catch((err) => {
        console.error('Failed to fetch templates:', err)
        setError('Template inventory could not be loaded.')
      })
      .finally(() => setLoading(false))
  }, [onChange])

  useEffect(() => {
    const handleClickOutside = (event) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
        setIsOpen(false)
      }
    }

    const handleEscape = (event) => {
      if (event.key === 'Escape') {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    document.addEventListener('keydown', handleEscape)

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
      document.removeEventListener('keydown', handleEscape)
    }
  }, [])

  const visibleTemplates = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return templates
    return templates.filter((template) => template.toLowerCase().includes(normalized))
  }, [query, templates])

  const persistSelection = useCallback(
    (nextSelection) => {
      onChange(nextSelection)
      localStorage.setItem(STORAGE_KEY, JSON.stringify(nextSelection))
    },
    [onChange]
  )

  const handleToggle = useCallback(
    (template) => {
      const selected = selectedTemplates.includes(template)
      const nextSelection = selected
        ? selectedTemplates.filter((item) => item !== template)
        : [...selectedTemplates, template]

      persistSelection(nextSelection)
    },
    [persistSelection, selectedTemplates]
  )

  const handleSelectAll = useCallback(() => {
    persistSelection(templates)
  }, [persistSelection, templates])

  const handleDeselectAll = useCallback(() => {
    persistSelection([])
  }, [persistSelection])

  return (
    <div className="template-selector" ref={dropdownRef}>
      <button
        type="button"
        className="template-dropdown-btn"
        onClick={() => setIsOpen((prev) => !prev)}
        disabled={disabled || loading}
        aria-expanded={isOpen}
      >
        <span className="template-button-copy">
          <strong>Detection templates</strong>
          <small>
            {loading
              ? 'Loading inventory…'
              : `${selectedTemplates.length} selected out of ${templates.length}`}
          </small>
        </span>
        <span className="dropdown-arrow">{isOpen ? 'Close' : 'Browse'}</span>
      </button>

      {isOpen && !loading && (
        <div className="template-dropdown">
          <div className="template-toolbar">
            <input
              type="text"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              className="template-search"
              placeholder="Filter templates"
            />
            <div className="template-actions">
              <button type="button" onClick={handleSelectAll} className="btn-action">
                Select all
              </button>
              <button type="button" onClick={handleDeselectAll} className="btn-action">
                Clear
              </button>
            </div>
          </div>

          {error ? (
            <div className="template-empty">{error}</div>
          ) : visibleTemplates.length === 0 ? (
            <div className="template-empty">No templates match the current filter.</div>
          ) : (
            <div className="template-list">
              {visibleTemplates.map((template) => (
                <label key={template} className="template-item">
                  <input
                    type="checkbox"
                    checked={selectedTemplates.includes(template)}
                    onChange={() => handleToggle(template)}
                  />
                  <span className="template-name">{template}</span>
                </label>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default TemplateSelector
