var L = Object.defineProperty;
var _ = (t, e, s) => e in t ? L(t, e, { enumerable: !0, configurable: !0, writable: !0, value: s }) : t[e] = s;
var u = (t, e, s) => _(t, typeof e != "symbol" ? e + "" : e, s);
import { ipcMain as l, app as c, BrowserWindow as P, dialog as w } from "electron";
import { fileURLToPath as R } from "node:url";
import r from "node:path";
import * as d from "fs/promises";
import { spawn as v } from "child_process";
import * as E from "fs";
const O = "/tmp/lex-desktop-lsp.log";
function p(t) {
  E.appendFileSync(O, `${(/* @__PURE__ */ new Date()).toISOString()} - ${t}
`);
}
class I {
  constructor() {
    u(this, "lspProcess", null);
    u(this, "webContents", null);
    this.setupIpc(), p("LspManager initialized");
  }
  setWebContents(e) {
    this.webContents = e;
  }
  start() {
    var s, i;
    if (this.lspProcess) return;
    const e = "/private/tmp/lex/desktop-app/target/debug/lex-lsp";
    console.log(`Spawning LSP from: ${e}`), p(`Spawning LSP from: ${e}`), this.lspProcess = v(e, [], {
      env: process.env
    }), (s = this.lspProcess.stdout) == null || s.on("data", (o) => {
      const a = o.toString();
      console.log(`LSP Output: ${a}`), p(`LSP Output: ${a}`), this.webContents && this.webContents.send("lsp-output", o);
    }), (i = this.lspProcess.stderr) == null || i.on("data", (o) => {
      const a = o.toString();
      console.error(`LSP Stderr: ${a}`), p(`LSP Stderr: ${a}`);
    }), this.lspProcess.on("exit", (o) => {
      console.log(`LSP exited with code ${o}`), p(`LSP exited with code ${o}`), this.lspProcess = null;
    }), this.lspProcess.on("error", (o) => {
      console.error("Failed to start LSP process:", o);
    });
  }
  setupIpc() {
    l.on("lsp-input", (e, s) => {
      this.lspProcess && this.lspProcess.stdin && this.lspProcess.stdin.write(s);
    });
  }
  stop() {
    this.lspProcess && (this.lspProcess.kill(), this.lspProcess = null);
  }
}
const m = r.dirname(R(import.meta.url));
process.env.APP_ROOT = r.join(m, "..");
const h = process.env.VITE_DEV_SERVER_URL, $ = r.join(process.env.APP_ROOT, "dist-electron"), g = r.join(process.env.APP_ROOT, "dist");
process.env.VITE_PUBLIC = h ? r.join(process.env.APP_ROOT, "public") : g;
let n;
const f = new I();
function S() {
  n = new P({
    icon: r.join(process.env.VITE_PUBLIC, "electron-vite.svg"),
    webPreferences: {
      preload: r.join(m, "preload.mjs")
    }
  }), f.setWebContents(n.webContents), f.start(), n.webContents.on("did-finish-load", () => {
    n == null || n.webContents.send("main-process-message", (/* @__PURE__ */ new Date()).toLocaleString());
  }), h ? n.loadURL(h) : n.loadFile(r.join(g, "index.html"));
}
l.handle("file-open", async () => {
  if (!n) return null;
  const { canceled: t, filePaths: e } = await w.showOpenDialog(n, {
    properties: ["openFile"],
    filters: [{ name: "Lex Files", extensions: ["lex"] }]
  });
  if (t || e.length === 0)
    return null;
  const s = e[0], i = await d.readFile(s, "utf-8");
  return { filePath: s, content: i };
});
l.handle("file-save", async (t, e, s) => (await d.writeFile(e, s, "utf-8"), !0));
l.handle("file-read-dir", async (t, e) => {
  try {
    return (await d.readdir(e, { withFileTypes: !0 })).map((i) => ({
      name: i.name,
      isDirectory: i.isDirectory(),
      path: r.join(e, i.name)
    }));
  } catch (s) {
    return console.error("Failed to read directory:", s), [];
  }
});
l.handle("file-read", async (t, e) => {
  try {
    return await d.readFile(e, "utf-8");
  } catch (s) {
    return console.error("Failed to read file:", s), null;
  }
});
l.handle("folder-open", async () => {
  if (!n) return null;
  const { canceled: t, filePaths: e } = await w.showOpenDialog(n, {
    properties: ["openDirectory"]
  });
  return t || e.length === 0 ? null : e[0];
});
c.on("window-all-closed", () => {
  f.stop(), process.platform !== "darwin" && (c.quit(), n = null);
});
c.on("activate", () => {
  P.getAllWindows().length === 0 && S();
});
c.whenReady().then(S);
export {
  $ as MAIN_DIST,
  g as RENDERER_DIST,
  h as VITE_DEV_SERVER_URL
};
