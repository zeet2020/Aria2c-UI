// Minimal browser EventEmitter shim — replaces node:events for the webview build.
// Covers the surface used by the shared aria2 JSON-RPC client:
// on / addListener / once / off / removeListener / removeAllListeners / emit / listeners.

export class EventEmitter {
  constructor () {
    this._events = Object.create(null)
  }

  on (type, listener) {
    (this._events[type] || (this._events[type] = [])).push(listener)
    return this
  }

  addListener (type, listener) {
    return this.on(type, listener)
  }

  once (type, listener) {
    const wrap = (...args) => {
      this.removeListener(type, wrap)
      listener.apply(this, args)
    }
    wrap.listener = listener
    return this.on(type, wrap)
  }

  removeListener (type, listener) {
    const list = this._events[type]
    if (!list) return this
    this._events[type] = list.filter(
      (l) => l !== listener && l.listener !== listener
    )
    return this
  }

  off (type, listener) {
    return this.removeListener(type, listener)
  }

  removeAllListeners (type) {
    if (type) {
      delete this._events[type]
    } else {
      this._events = Object.create(null)
    }
    return this
  }

  listeners (type) {
    return (this._events[type] || []).slice()
  }

  emit (type, ...args) {
    const list = this._events[type]
    if (!list || list.length === 0) return false
    list.slice().forEach((l) => l.apply(this, args))
    return true
  }
}

export default EventEmitter
