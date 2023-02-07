import { app, BrowserWindow, shell, ipcMain, dialog } from 'electron'
import { release } from 'node:os'
import { join, dirname } from 'node:path'
import { createReadStream, existsSync } from 'node:fs'
import { readFile, writeFile, mkdir, unlink } from 'node:fs/promises'
import * as readLine from 'node:readline'
import { exec } from 'node:child_process'
import { lookpath } from 'lookpath'

if (require('electron-squirrel-startup')) {
  app.quit();
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const homeDirectory = getHomeDirectory();
const cachePath = `${homeDirectory}/cache.json`;
const favsPath = `${homeDirectory}/favs.json`;
var mpvPath = "mpv";
var mpvProcesses = [];

fixMPV();

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

let win = null
const preload = MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY;
const url = "http://localhost:4200";
const indexHtml = join(__dirname, 'index.html');

async function createWindow() {
  win = new BrowserWindow({
    autoHideMenuBar: true,
    webPreferences: {
      preload,
      nodeIntegration: false,
      contextIsolation: true,
    },
  })

  if (process.env.DEVMODE) {
    win.loadURL(url);
  } else {
    win.loadURL(`file://${indexHtml}`);
  }

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
ipcMain.handle("deleteCache", deleteCache);
ipcMain.handle("saveFavs", async (event, favs) => saveFavs(favs));

async function deleteCache() {
  await unlink(cachePath);
  if (existsSync(favsPath))
    await unlink(favsPath);
}

async function selectFile() {
  let dialogResult = await dialog.showOpenDialog({ properties: ['openFile'] });
  if (dialogResult.canceled) return;
  let channels = await parsePlaylist(dialogResult.filePaths[0]);
  SaveToCache(channels);
  return channels;
}

async function getCache() {
  if (!existsSync(cachePath))
    return [];
  let cacheJson = await readFile(cachePath, { encoding: "utf-8" });
  let cache = JSON.parse(cacheJson);
  let favs = [];
  if (existsSync(favsPath)) {
    let favsJson = await readFile(favsPath, { encoding: "utf-8" });
    favs = JSON.parse(favsJson);
  }
  return { cache: cache, favs: favs };
}

async function SaveToCache(channels) {
  let json = JSON.stringify(channels);
  if (!existsSync(homeDirectory))
    mkdir(homeDirectory, { recursive: true });
  await writeFile(cachePath, json);
}

function getHomeDirectory() {
  let appdataPath = process.env.APPDATA ||
    (process.platform == 'darwin' ? process.env.HOME + '/Library/Preferences' :
      process.env.HOME + "/.local/share")
  return `${appdataPath}/open-tv`;
}

async function parsePlaylist(filePath) {
  const nameRegExp = /tvg-name="{1}(?<name>[^"]*)"{1}/;
  const logoRegExp = /tvg-logo="{1}(?<logo>[^"]*)"{1}/;
  const groupRegExp = /group-title="{1}(?<group>[^"]*)"{1}/;

  const inputStream = createReadStream(filePath);
  var lineReader = readLine.createInterface({
    input: inputStream,
    terminal: false,
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
      try {
        channels.push({
          name: firstLine.match(nameRegExp).groups.name,
          image: firstLine.match(logoRegExp).groups.logo,
          group: firstLine.match(groupRegExp).groups.group,
          url: secondLine
        });
      }
      catch (e) { }

      twoLines = [];
    }
  }
  return channels;
}

function clearMpvProcesses() {
  mpvProcesses.forEach(x => x.kill());
  mpvProcesses = [];
}
async function playChannel(url) {
  clearMpvProcesses();
  let child = await exec(`${mpvPath} ${url} --fs`);
  mpvProcesses.push(child);
  await waitForProcessStart(child);
}

function waitForProcessStart(proc) {
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

async function saveFavs(favs) {
  await writeFile(favsPath, JSON.stringify(favs));
}

async function fixMPV() {
  if (process.platform == 'darwin') {
    if (existsSync("/opt/homebrew/bin/mpv"))
      mpvPath = "/opt/homebrew/bin/mpv";
    else if (existsSync("/opt/local/mpv"))
      mpvPath = "/opt/local/mpv";
  }
  if (process.platform == "win32") {
    let mpvExists = await lookpath("mpv");
    if (!mpvExists) {
      mpvPath = join(__dirname, '..', 'libs', 'mpv.exe');
    }
  }
}