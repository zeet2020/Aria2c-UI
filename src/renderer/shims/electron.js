// Compatibility shim for `import { ipcRenderer } from 'electron'`.
// Maps Electron's renderer IPC onto Tauri's invoke/event system.
//
// Channel mapping:
//   ipcRenderer.invoke('get-app-config')      -> invoke('get_app_config')
//   ipcRenderer.send('command', cmd, ...args) -> invoke('dispatch_command', { command, args })
//   ipcRenderer.send('event', name, ...args)  -> invoke('dispatch_event', { name, args })
//   ipcRenderer.on('command', cb)             -> tauri listen('command') -> cb(evt, command, ...args)
//
// Rust handlers for dispatch_command / dispatch_event / get_app_config are
// implemented in src-tauri (see tasks: config store, window/tray, native ops).

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// channel name (kebab) -> tauri command name (snake)
const INVOKE_MAP = {
  'get-app-config': 'get_app_config'
}

// Track Tauri unlisten fns per channel so removeAllListeners can detach them.
const unlisteners = new Map()

function toCommand (channel) {
  return INVOKE_MAP[channel] || channel.replace(/-/g, '_')
}

export const ipcRenderer = {
  invoke (channel, ...args) {
    const cmd = toCommand(channel)
    // Pass a single positional arg through as `payload`; otherwise as `args`.
    const payload = args.length === 0
      ? undefined
      : args.length === 1
        ? args[0]
        : { args }
    return invoke(cmd, payload === undefined ? undefined : { payload })
  },

  send (channel, ...args) {
    if (channel === 'command') {
      const [command, ...rest] = args
      invoke('dispatch_command', { command, args: rest }).catch((e) =>
        console.warn('[bridge] dispatch_command failed:', command, e)
      )
      return
    }
    if (channel === 'event') {
      const [name, ...rest] = args
      invoke('dispatch_event', { name, args: rest }).catch((e) =>
        console.warn('[bridge] dispatch_event failed:', name, e)
      )
      return
    }
    // Generic fallthrough.
    invoke(toCommand(channel), { args }).catch((e) =>
      console.warn('[bridge] send failed:', channel, e)
    )
  },

  on (channel, listener) {
    // Tauri listen is async; store the promise-resolved unlisten for cleanup.
    const p = listen(channel, (event) => {
      const payload = event.payload || {}
      // Emulate (ipcEvent, ...args). Renderer code ignores the first arg.
      if (channel === 'command') {
        const { command, args = [] } = payload
        listener({}, command, ...args)
      } else {
        listener({}, payload)
      }
    })
    const list = unlisteners.get(channel) || []
    list.push(p)
    unlisteners.set(channel, list)
    return this
  },

  once (channel, listener) {
    const wrapped = (...a) => {
      this.removeListener(channel, wrapped)
      listener(...a)
    }
    return this.on(channel, wrapped)
  },

  removeAllListeners (channel) {
    const list = unlisteners.get(channel)
    if (!list) return this
    list.forEach((p) => p.then((un) => un && un()).catch(() => {}))
    unlisteners.delete(channel)
    return this
  },

  removeListener (channel) {
    // Tauri has no per-callback detach here; clear the channel.
    return this.removeAllListeners(channel)
  }
}

export default { ipcRenderer }
