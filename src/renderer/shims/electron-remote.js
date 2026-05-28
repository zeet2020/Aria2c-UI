// Compatibility shim for `@electron/remote`.
// Exposes the subset used by the renderer (dialog, shell, nativeTheme,
// getCurrentWindow, webContents, app), backed by Tauri plugins/APIs.

import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow as tauriCurrentWindow } from '@tauri-apps/api/window'
import { open as openDialog, ask as askDialog, message as messageDialog } from '@tauri-apps/plugin-dialog'
import { revealItemInDir, openPath as openerOpenPath, openUrl as openerOpenUrl } from '@tauri-apps/plugin-opener'

// ---- dialog ----
// Electron returns { canceled, filePaths }. Map onto plugin-dialog.open().
export const dialog = {
  // Electron showMessageBox -> { response: <buttonIndex> }. Mapped onto
  // Tauri ask() (2 buttons) / message() (1 button). Button[0] == "ok/yes".
  async showMessageBox (opts = {}) {
    const buttons = opts.buttons && opts.buttons.length ? opts.buttons : ['OK']
    const kind = opts.type === 'error'
      ? 'error'
      : opts.type === 'warning'
        ? 'warning'
        : 'info'
    if (buttons.length <= 1) {
      await messageDialog(opts.message || opts.title || '', { title: opts.title, kind })
      return { response: 0, checkboxChecked: false }
    }
    const ok = await askDialog(opts.message || opts.title || '', {
      title: opts.title,
      kind,
      okLabel: buttons[0],
      cancelLabel: buttons[1]
    })
    return { response: ok ? 0 : 1, checkboxChecked: false }
  },

  async showOpenDialog (opts = {}) {
    const properties = opts.properties || []
    const directory = properties.includes('openDirectory')
    const multiple = properties.includes('multiSelections')
    const selected = await openDialog({
      directory,
      multiple,
      defaultPath: opts.defaultPath,
      title: opts.title,
      filters: opts.filters
    })
    if (selected == null) {
      return { canceled: true, filePaths: [] }
    }
    const filePaths = Array.isArray(selected) ? selected : [selected]
    return { canceled: false, filePaths }
  }
}

// ---- shell ----
export const shell = {
  async showItemInFolder (fullPath) {
    return revealItemInDir(fullPath)
  },
  async openPath (fullPath) {
    try {
      await openerOpenPath(fullPath)
      return ''
    } catch (e) {
      return String(e)
    }
  },
  async openExternal (url) {
    return openerOpenUrl(url)
  },
  // No native "move to OS trash" in Tauri core; delegate to a Rust command.
  async trashItem (fullPath) {
    return invoke('trash_item', { path: fullPath })
  }
}

// ---- nativeTheme ----
// shouldUseDarkColors is read synchronously by the renderer; derive from
// the webview media query (kept in sync by the OS).
export const nativeTheme = {
  get shouldUseDarkColors () {
    return typeof window !== 'undefined' &&
      window.matchMedia &&
      window.matchMedia('(prefers-color-scheme: dark)').matches
  }
}

// ---- getCurrentWindow ----
// Wrap the Tauri window so the title-bar controls keep working. Note:
// isMaximized() is async in Tauri (TitleBar.vue is adapted accordingly).
export function getCurrentWindow () {
  const w = tauriCurrentWindow()
  return {
    minimize: () => w.minimize(),
    maximize: () => w.maximize(),
    unmaximize: () => w.unmaximize(),
    toggleMaximize: () => w.toggleMaximize(),
    isMaximized: () => w.isMaximized(),
    close: () => w.close(),
    hide: () => w.hide(),
    show: () => w.show(),
    setFullScreen: (v) => w.setFullscreen(v)
  }
}

// ---- webContents ----
// Electron in-app-browser introspection has no Tauri equivalent; stub so the
// Browser component loads. External-link handling is routed via ipc command.
export const webContents = {
  fromId: () => ({
    on: () => {},
    setWindowOpenHandler: () => {}
  })
}

// ---- app ----
// getVersion() is synchronous in Electron; cache the value at bootstrap.
let cachedVersion = ''
export async function __initAppVersion () {
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    cachedVersion = await getVersion()
  } catch (e) {
    console.warn('[bridge] failed to read app version:', e)
  }
}
export const app = {
  getVersion: () => cachedVersion,
  getName: () => 'Aria2 UI'
}

export default {
  dialog,
  shell,
  nativeTheme,
  getCurrentWindow,
  webContents,
  app
}
