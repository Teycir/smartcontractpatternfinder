import React, { useState, useEffect, useRef } from 'react'
import './TemplateSelector.css'

const TemplateSelector = ({ selectedTemplates, onChange, disabled }) => {
  const [templates, setTemplates] = useState([])
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef(null)

  useEffect(() => {
    fetch('http://127.0.0.1:8080/api/templates')
      .then(res => res.json())
      .then(data => {
        setTemplates(data.templates || [])
        if (!selectedTemplates.length) {
          onChange(data.templates || [])
        }
      })
      .catch(err => console.error('Failed to fetch templates:', err))
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

  const handleToggle = (template) => {
    const isSelected = selectedTemplates.includes(template)
    onChange(isSelected 
      ? selectedTemplates.filter(t => t !== template)
      : [...selectedTemplates, template]
    )
  }

  const handleSelectAll = () => {
    onChange(templates)
  }

  const handleDeselectAll = () => {
    onChange([])
  }

  return (
    <div className="template-selector" ref={dropdownRef}>
      <button
        className="template-dropdown-btn"
        onClick={() => setIsOpen(!isOpen)}
        disabled={disabled}
      >
        📋 Templates ({selectedTemplates.length}/{templates.length})
        <span className="dropdown-arrow">{isOpen ? '▲' : '▼'}</span>
      </button>
      {isOpen && (
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
