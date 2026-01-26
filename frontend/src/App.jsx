import React from 'react'
import Scanner from './components/Scanner'
import './App.css'

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <h1>🔍 SCPF - Smart Contract Pattern Finder</h1>
        <p>Detect vulnerabilities in smart contracts</p>
      </header>
      <main>
        <Scanner />
      </main>
    </div>
  )
}

export default App
