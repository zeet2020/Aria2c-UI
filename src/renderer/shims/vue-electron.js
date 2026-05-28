// Compatibility shim for `Vue.use(require('vue-electron'))`.
// The original injects Electron's API as `Vue.prototype.$electron`.
// Renderer code only uses `this.$electron.ipcRenderer`, so we expose the
// Tauri-backed ipcRenderer shim under the same name.

import { ipcRenderer } from './electron'

export default {
  install (app) {
    // Vue 3 plugin: expose on global properties so `this.$electron` works.
    app.config.globalProperties.$electron = { ipcRenderer }
  }
}
