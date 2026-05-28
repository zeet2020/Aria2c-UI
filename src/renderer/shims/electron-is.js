// Compatibility shim for `electron-is`.
// Electron-is is synchronous; we derive platform from the webview's
// navigator at load time (no async Tauri call needed for this).

const ua = (typeof navigator !== 'undefined' && navigator.userAgent) || ''
const plat = (typeof navigator !== 'undefined' && (navigator.userAgentData?.platform || navigator.platform)) || ''

const isMacOS = /Mac/i.test(plat) || /Macintosh/i.test(ua)
const isWindows = /Win/i.test(plat) || /Windows/i.test(ua)
const isLinux = /Linux/i.test(plat) && !/Android/i.test(ua)

const is = {
  macOS: () => isMacOS,
  osx: () => isMacOS,
  windows: () => isWindows,
  windows10: () => isWindows,
  linux: () => isLinux,
  // Process-role checks: in Tauri there is only the webview ("renderer").
  renderer: () => true,
  main: () => false,
  dev: () => Boolean(import.meta.env && import.meta.env.DEV),
  production: () => Boolean(import.meta.env && import.meta.env.PROD),
  // Misc helpers used sporadically; safe defaults.
  mas: () => false,
  windowsStore: () => false,
  all: (...args) => args.every(Boolean)
}

export default is
