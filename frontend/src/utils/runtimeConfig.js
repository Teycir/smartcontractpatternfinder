const DEFAULT_API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL?.trim() || window.location.origin

export const DEFAULT_RUNTIME_CONFIG = Object.freeze({
  service: 'scpf-runtime',
  apiBaseUrl: DEFAULT_API_BASE_URL,
  serverAddr: window.location.host || '',
  preferredEnvFile: '',
  envFiles: [],
  hasExplorerKeys: false,
  hasGithubToken: false,
})

let runtimeConfig = DEFAULT_RUNTIME_CONFIG
let runtimeConfigPromise = null

function getTauriInvoke() {
  if (typeof window === 'undefined') {
    return null
  }

  if (typeof window.__TAURI__?.core?.invoke === 'function') {
    return window.__TAURI__.core.invoke
  }

  if (typeof window.__TAURI_INTERNALS__?.invoke === 'function') {
    return window.__TAURI_INTERNALS__.invoke
  }

  return null
}

function normalizeRuntimeConfig(payload) {
  const apiBaseUrl = payload?.apiBaseUrl?.trim()

  return {
    ...DEFAULT_RUNTIME_CONFIG,
    ...payload,
    apiBaseUrl: apiBaseUrl || DEFAULT_RUNTIME_CONFIG.apiBaseUrl,
  }
}

async function resolveDesktopRuntimeConfig() {
  const invoke = getTauriInvoke()
  if (!invoke) {
    return null
  }

  try {
    const payload = await invoke('runtime_config')
    return normalizeRuntimeConfig(payload)
  } catch (error) {
    throw new Error(
      `Failed to resolve desktop runtime config: ${error?.message || String(error)}`
    )
  }
}

export async function loadRuntimeConfig() {
  if (!runtimeConfigPromise) {
    runtimeConfigPromise = (async () => {
      const desktopConfig = await resolveDesktopRuntimeConfig()
      runtimeConfig = desktopConfig || DEFAULT_RUNTIME_CONFIG
      return runtimeConfig
    })().catch((error) => {
      runtimeConfigPromise = null
      throw error
    })
  }

  return runtimeConfigPromise
}

export function getRuntimeConfigSync() {
  return runtimeConfig
}
