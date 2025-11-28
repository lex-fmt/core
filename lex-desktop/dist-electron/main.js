var S = Object.defineProperty;
var L = (t, e, s) => e in t ? S(t, e, { enumerable: !0, configurable: !0, writable: !0, value: s }) : t[e] = s;
var c = (t, e, s) => L(t, typeof e != "symbol" ? e + "" : e, s);
import { ipcMain as h, app as a, BrowserWindow as f, dialog as _ } from "electron";
import { fileURLToPath as R } from "node:url";
import i from "node:path";
import * as u from "fs/promises";
import { spawn as v } from "child_process";
import * as E from "fs";
const I = "/tmp/lex-desktop-lsp.log";
function l(t) {
  E.appendFileSync(I, `${(/* @__PURE__ */ new Date()).toISOString()} - ${t}
`);
}
class O {
  constructor() {
    c(this, "lspProcess", null);
    c(this, "webContents", null);
    this.setupIpc(), l("LspManager initialized");
  }
  setWebContents(e) {
    this.webContents = e;
  }
  start() {
    var s, p;
    if (this.lspProcess) return;
    const e = "/private/tmp/lex/desktop-app/target/debug/lex-lsp";
    console.log(`Spawning LSP from: ${e}`), l(`Spawning LSP from: ${e}`), this.lspProcess = v(e, [], {
      env: process.env
    }), (s = this.lspProcess.stdout) == null || s.on("data", (n) => {
      const r = n.toString();
      console.log(`LSP Output: ${r}`), l(`LSP Output: ${r}`), this.webContents && this.webContents.send("lsp-output", n);
    }), (p = this.lspProcess.stderr) == null || p.on("data", (n) => {
      const r = n.toString();
      console.error(`LSP Stderr: ${r}`), l(`LSP Stderr: ${r}`);
    }), this.lspProcess.on("exit", (n) => {
      console.log(`LSP exited with code ${n}`), l(`LSP exited with code ${n}`), this.lspProcess = null;
    }), this.lspProcess.on("error", (n) => {
      console.error("Failed to start LSP process:", n);
    });
  }
  setupIpc() {
    h.on("lsp-input", (e, s) => {
      this.lspProcess && this.lspProcess.stdin && this.lspProcess.stdin.write(s);
    });
  }
  stop() {
    this.lspProcess && (this.lspProcess.kill(), this.lspProcess = null);
  }
}
const w = i.dirname(R(import.meta.url));
process.env.APP_ROOT = i.join(w, "..");
const d = process.env.VITE_DEV_SERVER_URL, j = i.join(process.env.APP_ROOT, "dist-electron"), m = i.join(process.env.APP_ROOT, "dist");
process.env.VITE_PUBLIC = d ? i.join(process.env.APP_ROOT, "public") : m;
let o;
const P = new O();
function g() {
  o = new f({
    icon: i.join(process.env.VITE_PUBLIC, "electron-vite.svg"),
    webPreferences: {
      preload: i.join(w, "preload.mjs")
    }
  }), P.setWebContents(o.webContents), P.start(), o.webContents.on("did-finish-load", () => {
    o == null || o.webContents.send("main-process-message", (/* @__PURE__ */ new Date()).toLocaleString());
  }), d ? o.loadURL(d) : o.loadFile(i.join(m, "index.html"));
}
h.handle("file-open", async () => {
  if (!o) return null;
  const { canceled: t, filePaths: e } = await _.showOpenDialog(o, {
    properties: ["openFile"],
    filters: [{ name: "Lex Files", extensions: ["lex"] }]
  });
  if (t || e.length === 0)
    return null;
  const s = e[0], p = await u.readFile(s, "utf-8");
  return { filePath: s, content: p };
});
h.handle("file-save", async (t, e, s) => (await u.writeFile(e, s, "utf-8"), !0));
a.on("window-all-closed", () => {
  P.stop(), process.platform !== "darwin" && (a.quit(), o = null);
});
a.on("activate", () => {
  f.getAllWindows().length === 0 && g();
});
a.whenReady().then(g);
export {
  j as MAIN_DIST,
  m as RENDERER_DIST,
  d as VITE_DEV_SERVER_URL
};
