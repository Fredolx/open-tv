"use strict";
const electron = require("electron");
const node_os = require("node:os");
const node_path = require("node:path");
const node_fs = require("node:fs");
const promises = require("node:fs/promises");
const readLine = require("node:readline");
function _interopNamespaceDefault(e) {
  const n = Object.create(null, { [Symbol.toStringTag]: { value: "Module" } });
  if (e) {
    for (const k in e) {
      if (k !== "default") {
        const d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: () => e[k]
        });
      }
    }
  }
  n.default = e;
  return Object.freeze(n);
}
const readLine__namespace = /* @__PURE__ */ _interopNamespaceDefault(readLine);
process.env.DIST_ELECTRON = node_path.join(__dirname, "..");
process.env.DIST = node_path.join(process.env.DIST_ELECTRON, "../dist");
process.env.PUBLIC = process.env.VITE_DEV_SERVER_URL ? node_path.join(process.env.DIST_ELECTRON, "../public") : process.env.DIST;
if (node_os.release().startsWith("6.1"))
  electron.app.disableHardwareAcceleration();
if (process.platform === "win32")
  electron.app.setAppUserModelId(electron.app.getName());
if (!electron.app.requestSingleInstanceLock()) {
  electron.app.quit();
  process.exit(0);
}
let win = null;
const preload = node_path.join(__dirname, "../preload/index.js");
const url = process.env.VITE_DEV_SERVER_URL;
const indexHtml = node_path.join(process.env.DIST, "index.html");
async function createWindow() {
  win = new electron.BrowserWindow({
    title: "Open-TV (made by Frédéric Lachapelle)",
    icon: node_path.join(process.env.PUBLIC, "favicon.ico"),
    webPreferences: {
      preload,
      // Warning: Enable nodeIntegration and disable contextIsolation is not secure in production
      // Consider using contextBridge.exposeInMainWorld
      // Read more on https://www.electronjs.org/docs/latest/tutorial/context-isolation
      nodeIntegration: false,
      contextIsolation: true
    }
  });
  if (process.env.VITE_DEV_SERVER_URL) {
    win.loadURL(url);
  } else {
    win.loadFile(indexHtml);
  }
  win.webContents.setWindowOpenHandler(({ url: url2 }) => {
    if (url2.startsWith("https:"))
      electron.shell.openExternal(url2);
    return { action: "deny" };
  });
}
electron.app.whenReady().then(createWindow);
electron.app.on("window-all-closed", () => {
  win = null;
  if (process.platform !== "darwin")
    electron.app.quit();
});
electron.app.on("second-instance", () => {
  if (win) {
    if (win.isMinimized())
      win.restore();
    win.focus();
  }
});
electron.app.on("activate", () => {
  const allWindows = electron.BrowserWindow.getAllWindows();
  if (allWindows.length) {
    allWindows[0].focus();
  } else {
    createWindow();
  }
});
electron.ipcMain.handle("selectFile", selectFile);
electron.ipcMain.handle("getCache", getCache);
async function selectFile() {
  let dialogResult = await electron.dialog.showOpenDialog({ properties: ["openFile"] });
  if (dialogResult.canceled)
    return;
  let channels = await parsePlaylist(dialogResult.filePaths[0]);
  SaveToCache(channels);
  return channels;
}
async function getCache() {
  let cachePath = `${getHomeDirectory()}/cache.json`;
  if (!node_fs.existsSync(cachePath))
    return [];
  let json = await promises.readFile(cachePath, { encoding: "utf-8" });
  return JSON.parse(json);
}
async function SaveToCache(channels) {
  let json = JSON.stringify(channels);
  let path = getHomeDirectory();
  let cachePath = `${path}/cache.json`;
  if (!node_fs.existsSync(path))
    promises.mkdir(path, { recursive: true });
  await promises.writeFile(cachePath, json);
}
function getHomeDirectory() {
  let appdataPath = process.env.APPDATA || (process.platform == "darwin" ? process.env.HOME + "/Library/Preferences" : process.env.HOME + "/.local/share");
  return `${appdataPath}/open-tv`;
}
async function parsePlaylist(filePath) {
  const nameRegExp = /tvg-name="{1}(?<name>[^"]*)"{1}/;
  const logoRegExp = /tvg-logo="{1}(?<logo>[^"]*)"{1}/;
  const groupRegExp = /group-title="{1}(?<group>[^"]*)"{1}/;
  const inputStream = node_fs.createReadStream(filePath);
  var lineReader = readLine__namespace.createInterface({
    input: inputStream,
    terminal: false
  });
  let skippedFirstLine = false;
  let twoLines = [];
  let channels = [];
  for await (const line of lineReader) {
    if (!skippedFirstLine) {
      skippedFirstLine = true;
      continue;
    }
    twoLines.push(line);
    if (twoLines.length === 2) {
      let firstLine = twoLines[0];
      let secondLine = twoLines[1];
      channels.push({
        name: firstLine.match(nameRegExp)[0],
        image: firstLine.match(logoRegExp)[0],
        group: firstLine.match(groupRegExp)[0],
        url: secondLine
      });
      twoLines = [];
    }
  }
  return channels;
}
//# sourceMappingURL=index.js.map
