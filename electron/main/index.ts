import { app, BrowserWindow, shell, ipcMain, dialog } from 'electron'
import { release } from 'node:os'
import { join } from 'node:path'
import { createReadStream, existsSync } from 'node:fs'
import { readFile, open, writeFile, mkdir, unlink } from 'node:fs/promises'
import * as readLine from 'node:readline'
import { Channel } from '../../shared/dist/channel'
import { spawn } from 'node:child_process'
// The built directory structure
//
// ├─┬ dist-electron
// │ ├─┬ main
// │ │ └── index.js    > Electron-Main
// │ └─┬ preload
// │   └── index.js    > Preload-Scripts
// ├─┬ dist
// │ └── index.html    > Electron-Renderer
//
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const cacheFileName = "cache.json";
var mpvProcesses = [];

process.env.DIST_ELECTRON = join(__dirname, '..')
process.env.DIST = join(process.env.DIST_ELECTRON, '../dist')
process.env.PUBLIC = process.env.VITE_DEV_SERVER_URL
  ? join(process.env.DIST_ELECTRON, '../public')
  : process.env.DIST

// Disable GPU Acceleration for Windows 7
if (release().startsWith('6.1')) app.disableHardwareAcceleration()

// Set application name for Windows 10+ notifications
if (process.platform === 'win32') app.setAppUserModelId(app.getName())

if (!app.requestSingleInstanceLock()) {
  app.quit()
  process.exit(0)
}

// Remove electron security warnings
// This warning only shows in development mode
// Read more on https://www.electronjs.org/docs/latest/tutorial/security
// process.env['ELECTRON_DISABLE_SECURITY_WARNINGS'] = 'true'

let win: BrowserWindow | null = null
// Here, you can also use other preload
const preload = join(__dirname, '../preload/index.js')
const url = process.env.VITE_DEV_SERVER_URL
const indexHtml = join(process.env.DIST, 'index.html')

async function createWindow() {
  win = new BrowserWindow({
    title: 'Open-TV (made by Frédéric Lachapelle)',
    icon: join(process.env.PUBLIC, 'favicon.ico'),
    webPreferences: {
      preload,
      // Warning: Enable nodeIntegration and disable contextIsolation is not secure in production
      // Consider using contextBridge.exposeInMainWorld
      // Read more on https://www.electronjs.org/docs/latest/tutorial/context-isolation
      nodeIntegration: false,
      contextIsolation: true,
    },
  })

  if (process.env.VITE_DEV_SERVER_URL) {
    win.loadURL(url)
  } else {
    win.loadFile(indexHtml)
  }

  // Make all links open with the browser, not with the application
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('https:')) shell.openExternal(url)
    return { action: 'deny' }
  })
}

app.whenReady().then(createWindow)

app.on('window-all-closed', () => {
  win = null
  if (process.platform !== 'darwin') app.quit()
})

app.on('second-instance', () => {
  if (win) {
    // Focus on the main window if the user tried to open another
    if (win.isMinimized()) win.restore()
    win.focus()
  }
})

app.on('activate', () => {
  const allWindows = BrowserWindow.getAllWindows()
  if (allWindows.length) {
    allWindows[0].focus()
  } else {
    createWindow()
  }
})

ipcMain.handle("selectFile", selectFile);
ipcMain.handle("getCache", getCache);
ipcMain.handle("playChannel", async (event, url) => await playChannel(url));
ipcMain.handle("deleteCache", deleteCache)

async function deleteCache() {
  let cachePath = `${getHomeDirectory()}/${cacheFileName}`;
  await unlink(cachePath);
}

async function selectFile(): Promise<Array<Channel>> {
  let dialogResult = await dialog.showOpenDialog({ properties: ['openFile'] });
  if (dialogResult.canceled) return;
  let channels = await parsePlaylist(dialogResult.filePaths[0]);
  SaveToCache(channels);
  return channels;
}

async function getCache(): Promise<Array<Channel>> {
  let cachePath = `${getHomeDirectory()}/${cacheFileName}`;
  if (!existsSync(cachePath))
    return [];
  let json = await readFile(cachePath, { encoding: "utf-8" });
  return JSON.parse(json);
}

async function SaveToCache(channels: Array<Channel>) {
  let json = JSON.stringify(channels);
  let path = getHomeDirectory();
  let cachePath = `${path}/${cacheFileName}`
  if (!existsSync(path))
    mkdir(path, { recursive: true });
  await writeFile(cachePath, json);
}

function getHomeDirectory() {
  let appdataPath = process.env.APPDATA ||
    (process.platform == 'darwin' ? process.env.HOME + '/Library/Preferences' :
      process.env.HOME + "/.local/share")
  return `${appdataPath}/open-tv`;
}

async function parsePlaylist(filePath: string) {
  const nameRegExp = /tvg-name="{1}(?<name>[^"]*)"{1}/;
  const logoRegExp = /tvg-logo="{1}(?<logo>[^"]*)"{1}/;
  const groupRegExp = /group-title="{1}(?<group>[^"]*)"{1}/;

  const inputStream = createReadStream(filePath);
  var lineReader = readLine.createInterface({
    input: inputStream,
    terminal: false,
  });
  let skippedFirstLine = false;
  let twoLines: Array<string> = [];
  let channels: Array<Channel> = [];
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
        name: firstLine.match(nameRegExp).groups.name,
        image: firstLine.match(logoRegExp).groups.logo,
        group: firstLine.match(groupRegExp).groups.group,
        url: secondLine
      });
      twoLines = [];
    }
  }
  return channels;
}

async function playChannel(url: string) {
  mpvProcesses.forEach(x => x.kill());
  let child = await spawn('mpv', [url, "--fs"]);
  mpvProcesses.push(child);
  await waitForProcessStart(child);
}

function waitForProcessStart(proc): Promise<boolean> {
  return new Promise(function (resolve, reject) {
    const timeout = 20000;
    const timer = setTimeout(() => {
      reject(new Error(`Promise timed out after ${timeout} ms`));
    }, timeout);
    proc.stdout.on('data', function (data) {
      clearTimeout(timer);
      let line = data.toString();
      if (line.includes("AO") || line.includes("VO") || line.includes("AV")) {
        resolve(true);
      }
    });
    proc.on('close', function (code) {
      clearTimeout(timer);
      reject(code);
    });
  })
}