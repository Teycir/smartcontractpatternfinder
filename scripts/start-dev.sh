#!/bin/bash

# Start backend in background
echo "🚀 Starting backend server..."
cargo run --bin scpf-server &
BACKEND_PID=$!

# Wait for backend to be ready
sleep 3

# Start frontend
echo "🎨 Starting frontend..."
cd frontend && npm run dev

# Cleanup on exit
trap "kill $BACKEND_PID" EXIT
