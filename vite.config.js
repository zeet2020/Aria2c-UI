import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, pathToFileURL, URL } from 'node:url'

const r = (p) => fileURLToPath(new URL(p, import.meta.url))
// Forward-slash path for embedding in SCSS @import strings (backslashes are
// escape chars in sass, which breaks absolute Windows paths like D:\a\...).
const sassPath = (p) => r(p).replace(/\\/g, '/')

// Resolve webpack-style `~pkg` SCSS imports to node_modules (dart-sass legacy API).
const tildeImporter = (url) => {
  if (!url.startsWith('~')) return null
  return { file: r(`./node_modules/${url.slice(1)}`) }
}

// Tauri expects a fixed dev port; fail rather than silently pick another.
const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig({
  root: r('.'),
  // Relative base so the built assets load from tauri's asset protocol.
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': r('./src/renderer'),
      '@shared': r('./src/shared'),
      // Electron API -> Tauri compatibility shims (clean-cut: no Electron present).
      electron: r('./src/renderer/shims/electron.js'),
      'electron-is': r('./src/renderer/shims/electron-is.js'),
      'vue-electron': r('./src/renderer/shims/vue-electron.js'),
      '@electron/remote': r('./src/renderer/shims/electron-remote.js'),
      // Browser shims for node built-ins that leak in via the shared aria2 client.
      // The client already prefers global WebSocket/fetch; these stubs satisfy
      // the static imports so Vite can bundle without node polyfills.
      'node:events': r('./src/renderer/shims/events.js'),
      'node-fetch': r('./src/renderer/shims/node-empty.js'),
      ws: r('./src/renderer/shims/node-empty.js')
    },
    extensions: ['.mjs', '.js', '.vue', '.json', '.scss', '.css']
  },
  define: {
    // legacy code occasionally references `global`.
    global: 'globalThis',
    // shared/constants.js reads process.env.PORTABLE_EXECUTABLE_DIR at load;
    // the webview has no node `process`. Provide an empty env.
    'process.env': '{}',
    // vue-i18n@9 feature flags (silences bundler warnings, enables legacy API).
    __VUE_I18N_FULL_INSTALL__: 'true',
    __VUE_I18N_LEGACY_API__: 'true',
    __INTLIFY_PROD_DEVTOOLS__: 'false',
    // JIT compilation: compile locale messages to an AST and interpret them at
    // runtime instead of generating code via `new Function`. Avoids the CSP
    // `unsafe-eval` requirement so the strict `script-src 'self'` policy holds.
    __INTLIFY_JIT_COMPILATION__: 'true'
  },
  css: {
    preprocessorOptions: {
      scss: {
        // Point element-ui's icon @font-face at /fonts (served from public/),
        // set before its `!default` so the webview can load element-icons.
        // Then inject the theme variables (was webpack additionalData).
        additionalData: `$--font-path: '/fonts';\n@import "${sassPath('./src/renderer/components/Theme/Variables.scss')}";`,
        importer: [tildeImporter],
        // element-ui uses legacy sass syntax; silence the deprecation spam.
        quietDeps: true,
        silenceDeprecations: ['slash-div', 'global-builtin', 'import', 'legacy-js-api', 'function-units']
      }
    }
  },
  // --- Tauri-specific dev server settings ---
  clearScreen: false,
  server: {
    port: 9080,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 9081 } : undefined,
    watch: {
      // Don't watch the Rust side from the frontend dev server.
      ignored: ['**/src-tauri/**']
    }
  },
  build: {
    outDir: r('./dist/vite'),
    emptyOutDir: true,
    // WebView2 (Win) / WebKit (mac/linux) targets.
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    sourcemap: !!process.env.TAURI_ENV_DEBUG
  }
})
