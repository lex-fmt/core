var S = Object.defineProperty;
var g = (n, s, e) => s in n ? S(n, s, { enumerable: !0, configurable: !0, writable: !0, value: e }) : n[s] = e;
var c = (n, s, e) => g(n, typeof s != "symbol" ? s + "" : s, e);
import { ipcMain as f, app as p, BrowserWindow as h } from "electron";
import { fileURLToPath as L } from "node:url";
import r from "node:path";
import { spawn as _ } from "child_process";
import * as R from "fs";
const E = "/tmp/lex-desktop-lsp.log";
function l(n) {
  R.appendFileSync(E, `${(/* @__PURE__ */ new Date()).toISOString()} - ${n}
`);
}
class v {
  constructor() {
    c(this, "lspProcess", null);
    c(this, "webContents", null);
    this.setupIpc(), l("LspManager initialized");
  }
  setWebContents(s) {
    this.webContents = s;
  }
  start() {
    var e, P;
    if (this.lspProcess) return;
    const s = "/private/tmp/lex/desktop-app/target/debug/lex-lsp";
    console.log(`Spawning LSP from: ${s}`), l(`Spawning LSP from: ${s}`), this.lspProcess = _(s, [], {
      env: process.env
    }), (e = this.lspProcess.stdout) == null || e.on("data", (t) => {
      const i = t.toString();
      console.log(`LSP Output: ${i}`), l(`LSP Output: ${i}`), this.webContents && this.webContents.send("lsp-output", t);
    }), (P = this.lspProcess.stderr) == null || P.on("data", (t) => {
      const i = t.toString();
      console.error(`LSP Stderr: ${i}`), l(`LSP Stderr: ${i}`);
    }), this.lspProcess.on("exit", (t) => {
      console.log(`LSP exited with code ${t}`), l(`LSP exited with code ${t}`), this.lspProcess = null;
    }), this.lspProcess.on("error", (t) => {
      console.error("Failed to start LSP process:", t);
    });
  }
  setupIpc() {
    f.on("lsp-input", (s, e) => {
      this.lspProcess && this.lspProcess.stdin && this.lspProcess.stdin.write(e);
    });
  }
  stop() {
    this.lspProcess && (this.lspProcess.kill(), this.lspProcess = null);
  }
}
const m = r.dirname(L(import.meta.url));
process.env.APP_ROOT = r.join(m, "..");
const a = process.env.VITE_DEV_SERVER_URL, $ = r.join(process.env.APP_ROOT, "dist-electron"), w = r.join(process.env.APP_ROOT, "dist");
process.env.VITE_PUBLIC = a ? r.join(process.env.APP_ROOT, "public") : w;
let o;
const d = new v();
function u() {
  o = new h({
    icon: r.join(process.env.VITE_PUBLIC, "electron-vite.svg"),
    webPreferences: {
      preload: r.join(m, "preload.mjs")
    }
  }), d.setWebContents(o.webContents), d.start(), o.webContents.on("did-finish-load", () => {
    o == null || o.webContents.send("main-process-message", (/* @__PURE__ */ new Date()).toLocaleString());
  }), a ? o.loadURL(a) : o.loadFile(r.join(w, "index.html"));
}
p.on("window-all-closed", () => {
  d.stop(), process.platform !== "darwin" && (p.quit(), o = null);
});
p.on("activate", () => {
  h.getAllWindows().length === 0 && u();
});
p.whenReady().then(u);
export {
  $ as MAIN_DIST,
  w as RENDERER_DIST,
  a as VITE_DEV_SERVER_URL
};
