var L = Object.defineProperty;
var _ = (s, e, t) => e in s ? L(s, e, { enumerable: !0, configurable: !0, writable: !0, value: t }) : s[e] = t;
var u = (s, e, t) => _(s, typeof e != "symbol" ? e + "" : e, t);
import { ipcMain as l, app as c, BrowserWindow as P, dialog as w } from "electron";
import { fileURLToPath as R } from "node:url";
import r from "node:path";
import * as d from "fs/promises";
import { spawn as E } from "child_process";
import * as O from "fs";
const I = "/tmp/lex-desktop-lsp.log";
function p(s) {
  O.appendFileSync(I, `${(/* @__PURE__ */ new Date()).toISOString()} - ${s}
`);
}
class b {
  constructor() {
    u(this, "lspProcess", null);
    u(this, "webContents", null);
    this.setupIpc(), p("LspManager initialized");
  }
  setWebContents(e) {
    this.webContents = e;
  }
  start() {
    var t, i;
    if (this.lspProcess) return;
    const e = "/private/tmp/lex/desktop-app/target/debug/lex-lsp";
    console.log(`Spawning LSP from: ${e}`), p(`Spawning LSP from: ${e}`), this.lspProcess = E(e, [], {
      env: process.env
    }), (t = this.lspProcess.stdout) == null || t.on("data", (o) => {
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
    l.on("lsp-input", (e, t) => {
      this.lspProcess && this.lspProcess.stdin && this.lspProcess.stdin.write(t);
    });
  }
  stop() {
    this.lspProcess && (this.lspProcess.kill(), this.lspProcess = null);
  }
}
const m = r.dirname(R(import.meta.url));
process.env.APP_ROOT = r.join(m, "..");
const h = process.env.VITE_DEV_SERVER_URL, T = r.join(process.env.APP_ROOT, "dist-electron"), g = r.join(process.env.APP_ROOT, "dist");
process.env.VITE_PUBLIC = h ? r.join(process.env.APP_ROOT, "public") : g;
let n;
const f = new b();
function S() {
  n = new P({
    title: "Lex Editor",
    icon: r.join(process.env.VITE_PUBLIC, "icon.png"),
    webPreferences: {
      preload: r.join(m, "preload.mjs")
    }
  }), f.setWebContents(n.webContents), f.start(), n.webContents.on("did-finish-load", () => {
    n == null || n.webContents.send("main-process-message", (/* @__PURE__ */ new Date()).toLocaleString());
  }), h ? n.loadURL(h) : n.loadFile(r.join(g, "index.html"));
}
l.handle("file-open", async () => {
  if (!n) return null;
  const { canceled: s, filePaths: e } = await w.showOpenDialog(n, {
    properties: ["openFile"],
    filters: [{ name: "Lex Files", extensions: ["lex"] }]
  });
  if (s || e.length === 0)
    return null;
  const t = e[0], i = await d.readFile(t, "utf-8");
  return { filePath: t, content: i };
});
l.handle("file-save", async (s, e, t) => (await d.writeFile(e, t, "utf-8"), !0));
l.handle("file-read-dir", async (s, e) => {
  try {
    return (await d.readdir(e, { withFileTypes: !0 })).map((i) => ({
      name: i.name,
      isDirectory: i.isDirectory(),
      path: r.join(e, i.name)
    }));
  } catch (t) {
    return console.error("Failed to read directory:", t), [];
  }
});
l.handle("file-read", async (s, e) => {
  try {
    return await d.readFile(e, "utf-8");
  } catch (t) {
    return console.error("Failed to read file:", t), null;
  }
});
l.handle("folder-open", async () => {
  if (!n) return null;
  const { canceled: s, filePaths: e } = await w.showOpenDialog(n, {
    properties: ["openDirectory"]
  });
  return s || e.length === 0 ? null : e[0];
});
c.on("window-all-closed", () => {
  f.stop(), process.platform !== "darwin" && (c.quit(), n = null);
});
c.on("activate", () => {
  P.getAllWindows().length === 0 && S();
});
c.whenReady().then(S);
export {
  T as MAIN_DIST,
  g as RENDERER_DIST,
  h as VITE_DEV_SERVER_URL
};
