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
