import React, { useState, useEffect, useRef, useCallback } from 'react'
import './TemplateSelector.css'
import { fetchTemplates } from '../utils/api'

const TemplateSelector = ({ selectedTemplates, onChange, disabled }) => {
  const [templates, setTemplates] = useState([])
  const [isOpen, setIsOpen] = useState(false)
  const [loading, setLoading] = useState(true)
  const dropdownRef = useRef(null)
  const initializedRef = useRef(false)

  useEffect(() => {
    if (initializedRef.current) return
    initializedRef.current = true

    setLoading(true)
    fetchTemplates()
      .then(data => {
        const templateList = data.templates || []
        setTemplates(templateList)
        
        const saved = localStorage.getItem('selectedTemplates')
        if (saved) {
          try {
            const parsed = JSON.parse(saved)
            onChange(parsed.filter(t => templateList.includes(t)))
          } catch {
            onChange(templateList)
          }
        } else {
          onChange(templateList)
        }
      })
      .catch(err => console.error('Failed to fetch templates:', err))
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    const handleClickOutside = (e) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleToggle = useCallback((template) => {
    const isSelected = selectedTemplates.includes(template)
    const newSelection = isSelected 
      ? selectedTemplates.filter(t => t !== template)
      : [...selectedTemplates, template]
    onChange(newSelection)
    localStorage.setItem('selectedTemplates', JSON.stringify(newSelection))
  }, [selectedTemplates, onChange])

  const handleSelectAll = useCallback(() => {
    onChange(templates)
    localStorage.setItem('selectedTemplates', JSON.stringify(templates))
  }, [templates, onChange])

  const handleDeselectAll = useCallback(() => {
    onChange([])
    localStorage.setItem('selectedTemplates', JSON.stringify([]))
  }, [onChange])

  return (
    <div className="template-selector" ref={dropdownRef}>
      <button
        className="template-dropdown-btn"
        onClick={() => setIsOpen(!isOpen)}
        disabled={disabled || loading}
      >
        📋 Templates {loading ? '(Loading...)' : `(${selectedTemplates.length}/${templates.length})`}
        <span className="dropdown-arrow">{isOpen ? '▲' : '▼'}</span>
      </button>
      {isOpen && !loading && (
        <div className="template-dropdown">
          <div className="template-actions">
            <button onClick={handleSelectAll} className="btn-action">Select All</button>
            <button onClick={handleDeselectAll} className="btn-action">Deselect All</button>
          </div>
          <div className="template-list">
            {templates.map(template => (
              <label key={template} className="template-item">
                <input
                  type="checkbox"
                  checked={selectedTemplates.includes(template)}
                  onChange={() => handleToggle(template)}
                />
                <span>{template}</span>
              </label>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

export default TemplateSelector
