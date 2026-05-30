// Forward WebView console/runtime errors to the native log so they are visible
// in the terminal / log file even when the inspector isn't open. Install this
// before the app boots (see pages/index/main.js).
import { invoke } from '@tauri-apps/api/core'

// Keep raw refs so the forwarder never re-enters the patched console methods.
const rawError = console.error.bind(console)
const rawWarn = console.warn.bind(console)

function stringifyArg (arg) {
  // Must never throw: some values (Symbols, objects with a throwing
  // Symbol.toPrimitive, null-prototype objects) blow up String()/JSON.stringify
  // with "No default value" — which would re-enter the error path.
  try {
    if (arg instanceof Error) {
      return `${arg.name}: ${arg.message}\n${arg.stack || ''}`
    }
    if (typeof arg === 'symbol') {
      return arg.toString()
    }
    if (arg !== null && typeof arg === 'object') {
      try {
        return JSON.stringify(arg)
      } catch {
        return Object.prototype.toString.call(arg)
      }
    }
    return String(arg)
  } catch {
    try {
      return Object.prototype.toString.call(arg)
    } catch {
      return '[unserializable]'
    }
  }
}

function report (level, parts) {
  const message = parts.map(stringifyArg).join(' ')
  // Fire-and-forget; swallow bridge failures so logging can't crash the app.
  invoke('report_frontend_log', { level, message }).catch(() => {})
}

export function installErrorBridge () {
  // Uncaught exceptions.
  window.addEventListener('error', (e) => {
    if (e.error) {
      report('error', [e.error])
    } else {
      report('error', [`${e.message} (${e.filename}:${e.lineno}:${e.colno})`])
    }
  })

  // Unhandled promise rejections.
  window.addEventListener('unhandledrejection', (e) => {
    report('error', ['Unhandled rejection:', e.reason])
  })

  // Mirror console.error / console.warn to the native log (keep originals too).
  console.error = (...args) => {
    rawError(...args)
    report('error', args)
  }
  console.warn = (...args) => {
    rawWarn(...args)
    report('warn', args)
  }
}
