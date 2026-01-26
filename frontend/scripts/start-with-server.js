import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(__dirname, '..', '..');
const frontendRoot = join(__dirname, '..');

const SERVER_URL = 'http://127.0.0.1:8080/api/health';
const MAX_WAIT_SECONDS = 60;

async function checkServer() {
  try {
    const response = await fetch(SERVER_URL);
    return response.ok;
  } catch {
    return false;
  }
}

async function waitForServer(maxSeconds) {
  for (let i = 0; i < maxSeconds; i++) {
    if (await checkServer()) {
      return true;
    }
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
  return false;
}

async function startServer() {
  console.log('🚀 Starting backend server...');
  
  // Kill any existing server on port 8080
  try {
    const { execSync } = await import('child_process');
    execSync('lsof -ti:8080 | xargs kill -9 2>/dev/null || true', { stdio: 'ignore' });
    await new Promise(resolve => setTimeout(resolve, 500));
  } catch {
    // Ignore errors
  }
  
  const serverProcess = spawn('cargo', ['run', '--release', '--bin', 'scpf-server'], {
    cwd: projectRoot,
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: process.platform !== 'win32',
  });

  serverProcess.stdout.on('data', (data) => {
    process.stdout.write(`[server] ${data}`);
  });

  serverProcess.stderr.on('data', (data) => {
    process.stderr.write(`[server] ${data}`);
  });

  serverProcess.on('error', (err) => {
    console.error('❌ Failed to start server:', err.message);
    process.exit(1);
  });

  // Don't let the server process prevent exit
  serverProcess.unref();

  return serverProcess;
}

async function startVite() {
  console.log('🎨 Starting frontend...');
  
  const viteProcess = spawn('npx', ['vite'], {
    cwd: frontendRoot,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  viteProcess.on('error', (err) => {
    console.error('❌ Failed to start Vite:', err.message);
    process.exit(1);
  });

  viteProcess.on('close', (code) => {
    process.exit(code || 0);
  });
}

async function main() {
  console.log('🔍 Checking if server is running...');

  if (await checkServer()) {
    console.log('✅ Server already running');
  } else {
    await startServer();
    
    console.log('⏳ Waiting for server to be ready...');
    const ready = await waitForServer(MAX_WAIT_SECONDS);
    
    if (!ready) {
      console.error(`❌ Server failed to start within ${MAX_WAIT_SECONDS} seconds`);
      process.exit(1);
    }
    
    console.log('✅ Server is ready');
  }

  await startVite();
}

main().catch((err) => {
  console.error('❌ Error:', err.message);
  process.exit(1);
});
