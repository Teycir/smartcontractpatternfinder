# SCPF Web Application

This directory contains a React-based web interface for the Smart Contract Pattern Finder.

## Features

- **Start/Pause/Stop Controls**: Full control over the scanning process
- **Real-time Console**: Live output from the scan process via Server-Sent Events
- **Template Selection**: Choose which vulnerability templates to use with a dropdown checkbox selector
  - Select/deselect individual templates
  - Select all or deselect all with one click
  - Selection persisted in browser localStorage
  - Automatically syncs with available templates
- **Configurable Options**: All CLI options available through the UI with default settings
  - Contract addresses
  - Blockchain chain selection
  - Pages to fetch
  - Concurrency settings
  - Template filtering
  - Extract top riskiest sources
  - 0-day exploit fetching
  - And more...

## Architecture

- **Frontend**: React + Vite (port 3000)
- **Backend**: Rust Axum server (port 8080)
- **Communication**: REST API + Server-Sent Events (SSE) for real-time logs

## Getting Started

### 1. Install Frontend Dependencies

```bash
cd frontend
npm install
```

### 2. Start the Backend Server

```bash
cargo run --bin scpf-server
```

The server will start on `http://localhost:8080`

### 3. Start the Frontend

```bash
cd frontend
npm run dev
```

The web app will be available at `http://localhost:3000`

## Usage

1. Open your browser to `http://localhost:3000`
2. Configure scan options (or use defaults)
3. Click **Start** to begin scanning
4. Monitor progress in the real-time console
5. Use **Pause** or **Stop** to control the scan

## API Endpoints

- `GET /api/status` - Get current scan status and configuration
- `GET /api/templates` - Get list of available vulnerability templates
- `POST /api/start` - Start a new scan with provided configuration
- `POST /api/pause` - Pause the current scan
- `POST /api/resume` - Resume a paused scan
- `POST /api/stop` - Stop the current scan
- `GET /api/logs` - Server-Sent Events stream for real-time logs
- `POST /api/export-logs` - Export console logs to file

## Default Configuration

The UI uses the following defaults:

- **Chain**: all (Ethereum, Polygon, Arbitrum)
- **Pages**: 5
- **Concurrency**: 2
- **Extract Sources**: 50
- **Fetch 0-day**: enabled
- **Templates**: all templates selected by default

## Development

### Frontend

```bash
cd frontend
npm run dev      # Start development server
npm run build    # Build for production
npm run preview  # Preview production build
```

### Backend

```bash
cargo run --bin scpf-server              # Run in debug mode
cargo run --release --bin scpf-server    # Run in release mode
```

## File Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── Scanner.jsx           # Main scanner component with controls
│   │   ├── Scanner.css
│   │   ├── Console.jsx           # Real-time log console
│   │   ├── Console.css
│   │   ├── TemplateSelector.jsx  # Template dropdown with checkboxes
│   │   └── TemplateSelector.css
│   ├── constants/
│   │   └── index.js              # API endpoints and configuration
│   ├── utils/
│   │   ├── api.js                # API client functions
│   │   └── validation.js         # Input validation utilities
│   ├── App.jsx
│   ├── App.css
│   ├── main.jsx
│   └── index.css
├── .env                          # Environment variables (symlinked)
├── .env.example                  # Example environment configuration
├── index.html
├── vite.config.js
└── package.json

crates/scpf-server/
├── src/
│   └── main.rs                   # Axum backend server
└── Cargo.toml
```
