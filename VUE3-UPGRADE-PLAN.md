# Vue 2 → Vue 3 Upgrade Plan

Branch: `vue3-upgrade`. Scope: migrate the renderer (Vue 2.7 + Element-UI + Vuex 3 +
vue-router 3) to Vue 3 + Element Plus + Vuex 4 + vue-router 4, **keeping Options API**
(no `<script setup>` rewrite). The Tauri/Rust side is unaffected.

Estimated effort: **~10–16 dev-days**. Cost is dominated by Element Plus + the custom
theme; the rest is mechanical. **Add a test harness before starting** (zero coverage today).

---

## PROGRESS (uncommitted on `vue3-upgrade` — pending review)

**`vite build` is GREEN** at every step below. NOTE: green build ≠ runtime-correct —
the app has not been run yet; named slots / Element Plus component APIs / theme still need
runtime fixing.

DONE:
- **Deps**: package.json → vue@3, @vitejs/plugin-vue, element-plus, @element-plus/icons-vue,
  vue-router@4, vuex@4, vue-i18n@9; dropped vue@2/element-ui/@panter-vue-i18next/i18next/
  vuex-router-sync/vue-template-compiler. Installed.
- **Core bootstrap**: `main.js` → `createApp` + `app.use(...)` + ElementPlus(en locale, size small);
  `store` → `createStore`; `router` → `createRouter`+`createWebHashHistory`+`/:pathMatch(.*)*`;
  removed `vuex-router-sync` (`sync()` dropped — route-in-store was unused).
- **Shims**: `vue-electron` → Vue3 plugin (globalProperties); `Msg` → Vue3 plugin using `ElMessage`.
- **i18n**: `Locale/index.js` → `createI18n` (legacy mode), `getLocaleManager` shim kept; vite
  define flags added. `commands.js`/`native.js`/`Browser` Message/Loading → element-plus.
- **Filters (Phase 2) COMPLETE**: all 8 `filters:` blocks → `methods:`; ~20 `{{ x | f }}` →
  `{{ f(x) }}` (TaskFiles, TaskGeneral, TaskProgressInfo, TaskPeers, SelectTorrent, TaskActivity,
  TaskActions, Speedometer).
- **slot-scope (el-table)**: TaskFiles + TaskPeers `slot-scope="scope"` → `#default="scope"`.
- **SCSS**: removed the `~element-ui/.../theme-chalk` import from Index.scss (EP css via main.js).

DONE (cont.):
- **Named slots (Phase 3) COMPLETE**: all `slot="name"` → `<template #name>` (AddTask prepend/
  append/header/footer, Basic dir prepend/append, Advanced 4 dice + 3 show-in-folder, HistoryDirectory
  #reference, SubnavSwitcher #dropdown, TaskDetail/Index 5 tab #label, TaskGeneral #append). Dropped
  `.native` modifiers (AddTask @paste/@dragover/@drop).
- **Element Plus icons COMPLETE**: registered all `@element-plus/icons-vue` globally in main.js;
  converted every `el-icon-*` font class → `<el-icon><Comp/></el-icon>` (SubnavSwitcher ArrowDown,
  HistoryDirectory Star/StarFilled/Delete/Timer, TaskDetail/Index InfoFilled/Grid/Share/UserFilled/
  Files, TaskGeneral Link, AddTask Close).
- **Element Plus props**: dialogs `:visible`→`:model-value` (AddTask, AboutPanel, TaskDetail);
  `size="mini"`→`"small"`; `el-tabs :value`/`value`→`model-value` (AddTask, TaskDetail).

DONE (cont.):
- **Theme (Phase 6)**: Element Plus dark css-vars imported (`element-plus/theme-chalk/dark/css-vars.css`);
  App.vue toggles `html.dark` alongside `theme-dark`/`theme-light`; brand colour (#5b5bfa) mapped onto
  `--el-color-primary*` for light + dark in `Theme/ElementOverrides.scss`. The app's own custom
  Default/Dark.scss still layer app-specific styling (use the $-- sass vars, which remain valid values).
- **Minimal test (Phase 0)**: `test/utils.spec.js` (Vitest, 4 passing) + `npm test` script.

ALL phases structurally COMPLETE — `vite build` green, `npm test` green.

REMAINING (needs the running app — cannot be done headless):
- **Runtime QA (Phase 7)**: run `npx tauri dev` and walk every screen — dialog open/close (model-value
  is one-way + store-driven, as before EP), ThemeSwitcher live preview, el-table rendering/selection,
  tab switching, Add dialog (.txt upload + torrent), Preference save, tray/RPC. Fix any per-component
  Element Plus behaviour diffs found at runtime (theme tint tuning, dialog header padding, etc.).

How to run this branch:
```
npm install                 # .npmrc has legacy-peer-deps
npx tauri dev               # rebuilds Rust + serves the Vue 3 renderer
```

---

## Codebase facts (scanned)

| Pattern | Count | Action |
|---|---|---|
| `.vue` components | 45 | all re-tested |
| Template filters `{{ x \| f }}` | ~252 / 8 `filters:` blocks | **removed in Vue 3** → convert to methods/computed |
| Old slot syntax (`slot=`, `slot-scope`) | 31 | → `v-slot` |
| `new Vue()` | 1 (`main.js`) | → `createApp()` |
| `Vue.use/component/prototype/config` | ~13 | → `app.use/component/config.globalProperties` |
| Element-UI (`el-*`) | 33 distinct tags / 45 files | → Element Plus |
| Element services (Message/MessageBox/Loading) | 19 | → ElMessage/ElMessageBox/ElLoading |
| event bus `$on`, `$children/$parent`, mixins, `$scopedSlots`, `$set/$delete`, `require.context` | **0** | ✅ none — already clean / fixed |

No event bus, no mixins, no `$children` — keeps this from being worse. `require.context`
was already migrated to `import.meta.glob` during the Tauri port.

---

## Dependency changes

**Remove**: `vue@2.7`, `vue-template-compiler`, `@vitejs/plugin-vue2`, `element-ui`,
`vue-router@3`, `vuex@3`, `vuex-router-sync@5`, `@panter/vue-i18next`, `i18next@22` (re-decide).

**Add**: `vue@3`, `@vitejs/plugin-vue`, `element-plus`, `@element-plus/icons-vue`,
`vue-router@4`, `vuex@4` (or migrate to `pinia`), `vue-i18n@9` (or `vue-i18next@3`),
`vuex-router-sync@6` (or drop — small).

`vite.config.js`: swap `@vitejs/plugin-vue2` → `@vitejs/plugin-vue`; drop the
`vue$ → vue/dist/vue.esm.js` alias (Vue 3 uses runtime + the render function in `main.js`).
Keep the `~` sass importer, `@`/`@shared` aliases, polyfills, `process.env`/`global` defines,
and the `$--font-path` / `public/fonts` setup until the theme is re-done.

---

## Phased plan

### Phase 0 — Safety net (do first)
- Add a test runner (Vitest + @vue/test-utils@2) and a thin smoke test per major view.
- Wire `lint`/typecheck to CI on this branch. Tag a known-good baseline.

### Phase 1 — Core bootstrap (small, ~1 day)
- `main.js`: `new Vue({...}).$mount('#app')` → `createApp(App)`, `app.use(router)`,
  `app.use(store)`, `app.use(i18n)`, `app.mount('#app')`. Already uses `render: h => h(App)` —
  becomes the root component directly.
- `Vue.prototype.$http`/`$electron` → `app.config.globalProperties.$http`/`$electron`.
  Rewrite the `vue-electron` shim as a Vue 3 plugin (`install(app){ app.config.globalProperties.$electron = ... }`).
- `Vue.use(Msg, ...)`, `Vue.component('mo-icon', Icon)` → `app.use` / `app.component`.
- Drop `Vue.config.productionTip` (gone).
- `vuex` → `createStore`; `vue-router` → `createRouter({ history: createWebHashHistory() })`;
  fix catch-all route `path: '*'` → `path: '/:pathMatch(.*)*'`. Keep **hash** history (Tauri asset protocol).
- `vuex-router-sync@6` or remove.

### Phase 2 — Filters → methods (medium, ~1–2 days)
- Remove all `filters: { ... }` blocks; expose the same fns as methods or shared helpers
  (e.g. `bytesToSize`, `removeExtensionDot`). Replace ~252 `{{ x | f }}` with `{{ f(x) }}`.
- Mechanical and low-risk; do per-component, lean on the test net.

### Phase 3 — Slots + template syntax (small, ~0.5 day)
- 31 `slot=`/`slot-scope` → `v-slot:` / `#name`. Check `<template>` wrappers.
- Vue 3 v-model/`.sync` and event changes: audit `.native` modifiers (removed) and any
  `$listeners` (none here). `@drop.native`/`@paste.native` (AddTask) → drop `.native`.

### Phase 4 — i18n swap (medium, ~0.5–1 day)
- Replace `@panter/vue-i18next` with `vue-i18n@9` (Composition/Legacy mode) or `vue-i18next@3`.
- Re-wire `LocaleManager` + `getLocaleManager()` + the `$t` provider. English-only already, so
  the resource surface is small (`en-US/{app,preferences,subnav,task}`). Element locale via
  Element Plus's own i18n (`ElConfigProvider` / `locale`).

### Phase 5 — Element-UI → Element Plus (LARGE, ~3–5 days)
- Replace `element-ui` import/registration. Prefer `unplugin-vue-components` +
  `unplugin-auto-import` with the Element Plus resolver (per-component auto-import) over global.
- **Icons**: `el-icon-*` font classes are gone → `<el-icon><Close/></el-icon>` from
  `@element-plus/icons-vue`. Audit every Element icon usage. Custom `mo-icon` SVG sprite is unaffected.
- **Services**: `this.$message/$msgbox/$confirm`, `Loading.service`, `MessageBox` →
  `ElMessage`/`ElMessageBox`/`ElLoading`. Rewrite the `Msg` plugin wrapper.
- **Props/events**: `size: 'mini'` removed → `small` (global density shift, visual re-check);
  `el-submenu` → `el-sub-menu`; some `el-*` prop/event renames — go component-by-component.
- The element-icons `$--font-path`/`public/fonts` hack from the Tauri port becomes obsolete
  (Element Plus ships its own icon components) — remove it.

### Phase 6 — Theme rewrite (LARGE / most uncertain, ~2–4 days)
- The custom theme (`Theme/Variables.scss` ~1000 lines + `Default/Dark/Light.scss`) is built on
  Element-UI's `$--xxx` SCSS variable system, which **Element Plus removed** (now namespaced CSS
  custom properties + a different SCSS API).
- Re-author: light/dark theming against Element Plus's CSS-var model; keep the app's own
  component styles (those are independent). The live theme-switch (`document.documentElement.className`
  → `.theme-dark`) logic can stay; only the variable plumbing changes.
- Verify the `tildeImporter` + sass `additionalData` injection still make sense (likely simplified).

### Phase 7 — QA (medium-high, ~2–3 days)
- Manual pass over every screen (Aside nav, Task list/detail, Add dialog incl. the new `.txt`
  upload + torrent, all Preference panels, About). Verify dialogs, drag regions
  (`data-tauri-drag-region`), theme toggle, RPC flow (add/pause/resume/delete).
- Run under `tauri dev` (WebKitGTK/WebView2), not just a browser — webview quirks differ.

---

## Risks / watch-list
- **Element Plus visual + behavioral drift** across 33 component types → QA-heavy (no tests today).
- **Theme system rewrite** is the biggest unknown; budget conservatively.
- **`vue-i18next` (Vue2-only) is dead** → must switch i18n libs; verify `$t` key compatibility.
- Vue 3 reactivity: arrays/objects are deeply reactive (no `$set`) — should *simplify* code, but
  re-check any index-based mutations in the Vuex `task` module.
- Keep **hash router history** (Tauri serves via asset protocol; HTML5 history needs a fallback).
- The Tauri shims (`electron`, `electron-is`, `@electron/remote`, `vue-electron`) — only
  `vue-electron` needs Vue-3 plugin changes; the rest are framework-agnostic.

## Sequence summary
`0 safety net → 1 core → 2 filters → 3 slots → 4 i18n → 5 Element Plus → 6 theme → 7 QA`.
Land each phase as its own commit on `vue3-upgrade`; the app won't fully run until Phase 5+ is
coherent, so expect a long-lived branch — keep `main` shippable.
