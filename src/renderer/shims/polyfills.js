// Minimal node-global polyfills for the few browser-hostile libraries the
// renderer still pulls in (parse-torrent -> bencode need Buffer; some expect
// a `process` object). Imported first in the entry, before anything else.
import { Buffer } from 'buffer'

if (typeof globalThis.Buffer === 'undefined') {
  globalThis.Buffer = Buffer
}

if (typeof globalThis.process === 'undefined') {
  globalThis.process = {
    env: {},
    browser: true,
    version: '',
    nextTick: (fn, ...args) => Promise.resolve().then(() => fn(...args))
  }
}

// `setImmediate` is a Node global with no WebKit/WebView equivalent; several
// components (TaskActivity, TaskDetail, AddTask) call it. Map it to a macrotask.
if (typeof globalThis.setImmediate === 'undefined') {
  globalThis.setImmediate = (fn, ...args) => setTimeout(fn, 0, ...args)
}
if (typeof globalThis.clearImmediate === 'undefined') {
  globalThis.clearImmediate = (id) => clearTimeout(id)
}
