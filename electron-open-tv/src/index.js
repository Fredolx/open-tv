import { app, BrowserWindow, shell, ipcMain, dialog } from 'electron'
import { release, homedir, platform } from 'node:os'
import { join, dirname } from 'node:path'
import { createReadStream, existsSync, mkdirSync, writeSync } from 'node:fs'
import { readFile, writeFile, mkdir, unlink } from 'node:fs/promises'
import * as readLine from 'node:readline'
import { exec } from 'node:child_process'
import { lookpath } from 'lookpath'
import axios from 'axios'
import { nameRegExp, idRegExp, logoRegExp, groupRegExp } from './regExps'
import { stringify } from 'node:querystring'

if (require('electron-squirrel-startup')) {
  app.quit();
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const appDataPath = getAppDataPath();
const cachePath = join(appDataPath, "cache.json");
const favsPath = join(appDataPath, "favs.json");
const settingsPath = join(appDataPath, "settings.json");
const homePath = homedir();
const defaultRecordingPath = getVideosPath();
var m3uURL;
var settings;
var mpvPath = "mpv";
var mpvProcesses = [];

fetchSettings();
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
      devTools: process.env.DEVMODE ?? false
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
  app.quit()
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
ipcMain.handle("playChannel", async (event, url, record) => await playChannel(url, record));
ipcMain.handle("deleteCache", deleteCache);
ipcMain.handle("saveFavs", async (event, favs) => saveFavs(favs));
ipcMain.handle("downloadM3U", async (event, url) => await downloadM3U(url));
ipcMain.handle("selectFolder", selectFolder);
ipcMain.handle("updateSettings", async (event, settings) => await updateSettings(settings));
ipcMain.handle("getSettings", getSettings);

async function updateSettings(_settings) {
  settings = _settings;
  let json = JSON.stringify(settings);
  await writeFile(settingsPath, json);
}

async function selectFolder() {
  let dialogResult = await dialog.showOpenDialog({ properties: ['openDirectory'] });
  if (dialogResult.canceled) return null;
  return dialogResult.filePaths[0];
}

async function deleteCache() {
  await unlink(cachePath);
  if (existsSync(favsPath))
    await unlink(favsPath);
}

async function selectFile() {
  let dialogResult = await dialog.showOpenDialog({ properties: ['openFile'] });
  if (dialogResult.canceled) return;
  this.m3uURL = null;
  let channels = await parsePlaylist(dialogResult.filePaths[0]);
  await SaveToCache(channels);
  return channels;
}

async function downloadM3U(url) {
  let result;
  try {
    result = await axios.get(url);
  }
  catch (e) {
    console.error(e);
    return [];
  }
  m3uURL = url;
  let channels = parsePlaylistFromMemory(result.data.split("\n"));
  await SaveToCache(channels, url);
  return channels;
}

function getVideosPath() {
  let vPath;
  if (process.platform == 'darwin')
    vPath = join(homePath, 'Movies', 'open-tv');
  else
    vPath = join(homePath, 'Videos', 'open-tv');
  if (!existsSync(vPath))
    mkdirSync(vPath);
  return vPath;
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
  if (cache?.url?.trim())
    m3uURL = cache.url;
  return { cache: cache, favs: favs, settings };
}

async function getSettings() {
  return settings;
}

async function fetchSettings() {
  if (existsSync(settingsPath))
    settings = JSON.parse(await readFile(settingsPath, { encoding: "utf-8" }));
}

async function SaveToCache(channels, url = null) {
  let json = JSON.stringify({ channels: channels, url: url });
  if (!existsSync(appDataPath))
    mkdir(appDataPath, { recursive: true });
  await writeFile(cachePath, json);
}

function getAppDataPath() {
  let appdataPath = process.env.APPDATA ||
    (process.platform == 'darwin' ? process.env.HOME + '/Library/Preferences' :
      process.env.HOME + "/.local/share")
  return `${appdataPath}/open-tv`;
}

function processChannel(twoLines) {
  let firstLine = twoLines[0];
  let secondLine = twoLines[1];
  try {
    let channel = {
      name: firstLine.match(nameRegExp)?.groups?.name,
      image: firstLine.match(logoRegExp)?.groups?.logo,
      group: firstLine.match(groupRegExp)?.groups?.group,
      url: secondLine.trim()
    }
    if (!channel.name || !channel.name?.trim())
      channel.name = firstLine.match(idRegExp)?.groups?.id;

    if (channel.name && channel.name?.trim() && channel.url && channel.url?.trim()) {
      return channel;
    }
  }
  catch (e) { }
  return null;
}

function parsePlaylistFromMemory(lines) {
  let twoLines = [];
  let channels = [];
  lines.shift();
  lines.forEach(line => {
    twoLines.push(line);
    if (twoLines.length == 2) {
      let channel = processChannel(twoLines);
      if (channel)
        channels.push(channel);
      twoLines = [];
    }
  });
  return channels;
}

async function parsePlaylist(filePath) {
  const inputStream = createReadStream(filePath);
  var lineReader = readLine.createInterface({
    input: inputStream,
    terminal: false,
  });
  let twoLines = [];
  let channels = [];
  await lineReader[Symbol.asyncIterator]().next();
  for await (const line of lineReader) {
    twoLines.push(line);
    if (twoLines.length == 2) {
      let channel = processChannel(twoLines);
      if (channel)
        channels.push(channel);
      twoLines = [];
    }
  }
  return channels;
}

function clearMpvProcesses() {
  mpvProcesses.forEach(x => x.kill());
  mpvProcesses = [];
}

async function playChannel(url, record) {
  clearMpvProcesses();
  let command = `${mpvPath} ${url} --fs`
  if (url.endsWith(".mp4") || url.endsWith(".mkv"))
    command += " --save-position-on-quit";
  else if (record === true) {
    let recordingFilePath = join(settings?.recordingPath ?? defaultRecordingPath, getRecordingFileName());
    command += ` --stream-record="${recordingFilePath}"`
  }
  let child = await exec(command);
  mpvProcesses.push(child);
  console.log("Waiting for mpv start");
  await waitForProcessStart(child);
  console.log(`Playing channel from URL: ${url}`);
}

function getRecordingFileName() {
  let date = new Date();
  let month = ("0" + (date.getMonth() + 1)).slice(-2);
  return `${date.getFullYear()}-${month}-${date.getHours()}-${date.getMinutes()}.mp4`;
}

function waitForProcessStart(proc) {
  return new Promise(function (resolve, reject) {
    const timeout = 10000;
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