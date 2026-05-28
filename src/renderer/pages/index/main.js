import '@/shims/polyfills'
import { createApp } from 'vue'
import ElementPlus, { ElLoading } from 'element-plus'
import * as ElementPlusIcons from '@element-plus/icons-vue'
import enLocale from 'element-plus/es/locale/lang/en'
import axios from 'axios'

import App from './App'
import router from '@/router'
import store from '@/store'
import VueElectron from 'vue-electron'
import { i18n } from '@/components/Locale'
import Icon from '@/components/Icons/Icon'
import Msg from '@/components/Msg'
import { commands } from '@/components/CommandManager/instance'
import { __initAppVersion } from '@/shims/electron-remote'

import 'normalize.css'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import '@/components/Theme/Index.scss'

// Disable the webview context menu (and any "Inspect") in production builds.
// Kept enabled in dev for debugging. Right-click still works inside text fields.
if (import.meta.env.PROD) {
  window.addEventListener('contextmenu', (e) => {
    const el = e.target
    const editable =
      el && (el.isContentEditable ||
        el.tagName === 'INPUT' ||
        el.tagName === 'TEXTAREA')
    if (!editable) {
      e.preventDefault()
    }
  })
}

function init () {
  const app = createApp(App)

  app.config.globalProperties.$http = axios

  app.use(router)
  app.use(store)
  app.use(i18n)
  app.use(ElementPlus, { size: 'small', locale: enLocale })
  // Register all Element Plus icons globally (e.g. <el-icon><ArrowDown/></el-icon>).
  for (const [name, comp] of Object.entries(ElementPlusIcons)) {
    app.component(name, comp)
  }
  app.use(VueElectron)
  app.use(Msg, { showClose: true })
  app.component('mo-icon', Icon)

  app.config.globalProperties.$commands = commands
  window.app = app

  const loading = ElLoading.service({
    fullscreen: true,
    background: 'rgba(0, 0, 0, 0.1)'
  })

  app.mount('#app')

  // Side-effect import: registers all application:* commands.
  import('./commands')

  setTimeout(() => {
    loading.close()
  }, 400)
}

// Cache the app version (Electron's app.getVersion was synchronous).
__initAppVersion()

store.dispatch('preference/fetchPreference')
  .then((config) => {
    console.info('[AUI] load preference:', config)
    init(config)
  })
  .catch((err) => {
    alert(err)
  })
