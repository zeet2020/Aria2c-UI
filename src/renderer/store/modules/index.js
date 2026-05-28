/**
 * Imports all vuex modules in this directory in one shot.
 * Uses Vite's import.meta.glob (replaces webpack's require.context).
 */

const files = import.meta.glob('./*.js', { eager: true })
const modules = {}

Object.keys(files).forEach(key => {
  if (key === './index.js') return
  const name = key.replace(/(\.\/|\.js)/g, '')
  modules[name] = files[key].default
})

export default modules
