// Empty stub for node-only modules (node-fetch, ws) that are statically imported
// by the shared aria2 client but never used in the webview: the client prefers
// the browser-native global WebSocket / fetch. See JSONRPCClient.js.
export default undefined
