import { app, BrowserWindow, shell, ipcMain, dialog } from 'electron'
import { release, homedir } from 'node:os'
import { join } from 'node:path'
import { createReadStream, existsSync, mkdirSync } from 'node:fs'
import { readFile, writeFile, mkdir, unlink } from 'node:fs/promises'
import * as readLine from 'node:readline'
import { exec } from 'node:child_process'
import { lookpath } from 'lookpath'
import axios from 'axios'
import { nameRegExp, idRegExp, logoRegExp, groupRegExp } from './regExps'
import { getLiveStreams, getSeries, getVods } from './xtreamActions'
import { live, series, vods } from './xtreamStreamTypes'
import { livestream, movie, serie } from './mediaType'

if (require('electron-squirrel-startup')) {
  app.quit();
}

if (!app.requestSingleInstanceLock()) {
  app.quit();
  process.exit(0);
}

const appDataPath = getAppDataPath();
const cachePath = join(appDataPath, "cache.json");
const favsPath = join(appDataPath, "favs.json");
const settingsPath = join(appDataPath, "settings.json");
const homePath = homedir();
const defaultRecordingPath = getVideosPath();
var settings = {};
var mpvPath = "mpv";
var mpvProcesses = [];

fetchSettings();
fixMPV();

// Disable GPU Accel in Windows 7
if (release().startsWith('6.1'))
  app.disableHardwareAcceleration();

// Windows 10/11 notifications requirement to get proper name
if (process.platform === 'win32')
  app.setAppUserModelId(app.getName());

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

app.whenReady().then(createWindow);

app.on('window-all-closed', () => {
  win = null
  app.quit()
});

app.on('second-instance', () => {
  if (win) {
    // Refocus the main window if user tries to open a new instance
    if (win.isMinimized()) win.restore()
    win.focus()
  }
});

app.on('activate', () => {
  const allWindows = BrowserWindow.getAllWindows()
  if (allWindows.length) {
    allWindows[0].focus();
  } else {
    createWindow();
  }
});

ipcMain.handle("selectFile", selectFile);
ipcMain.handle("getCache", getCache);
ipcMain.handle("playChannel", async (_, url, record) => await playChannel(url, record));
ipcMain.handle("deleteCache", deleteCache);
ipcMain.handle("saveFavs", async (_, favs) => saveFavs(favs));
ipcMain.handle("downloadM3U", async (_, url) => await downloadM3U(url));
ipcMain.handle("selectFolder", selectFolder);
ipcMain.handle("updateSettings", async (_, settings) => await updateSettings(settings));
ipcMain.handle("getSettings", getSettings);
ipcMain.handle("getXtream", async (_, xtream) => await getXtream(xtream));

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
  let channels = await parsePlaylist(dialogResult.filePaths[0]);
  await saveToCache({ channels: channels });
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
  let channels = parsePlaylistFromMemory(result.data.split("\n"));
  await saveToCache({ channels: channels, url: url });
  return channels;
}

async function getXtream(xtream) {
  let url = new URL(xtream.url);
  url.searchParams.append('username', xtream.username);
  url.searchParams.append('password', xtream.password);
  let reqs = [];
  let responses;
  try {
    url.searchParams.append('action', getLiveStreams);
    reqs.push(axios.get(url.toString()));

    url.searchParams.set('action', getVods)
    reqs.push(axios.get(url.toString()));

    //@TODO different impl for series
    // url.searchParams.set('action', getSeries)
    // reqs.push(axios.get(url.toString()));

    responses = await Promise.all(reqs);
  }
  catch (e) {
    console.error(e);
    return [];
  }
  let channels = [];
  responses.forEach(x => {
    channels = channels.concat(parseXtreamResponse(x.data, xtream));
  });
  await saveToCache(
    {
      channels: channels,
      username: xtream.username,
      password: xtream.password,
      url: xtream.url
    });
  return channels;
}

function parseXtreamResponse(data, xtream) {
  let baseURL = new URL(xtream.url).origin;
  let channels = [];
  data.forEach(x => channels.push(xtreamToChannel(x, baseURL, xtream.username, xtream.password)));
  return channels;
}

function xtreamToChannel(xtreamChannel, baseURL, username, password) {
  return {
    url: `${baseURL}/${xtreamChannel.stream_type}/${username}/${password}/${xtreamChannel.stream_id}.${xtreamChannel.stream_type.toLowerCase().trim() == live ? 'ts' : xtreamChannel.container_extension}`,
    name: xtreamChannel.name,
    image: xtreamChannel.stream_icon,
    group: xtreamChannel.category_id,
    type: xtreamChannel.stream_type == live ? livestream : movie
  }
}

function getVideosPath() {
  let vPath;
  if (process.platform == 'darwin')
    vPath = join(homePath, 'Movies', 'open-tv');
  else
    vPath = join(homePath, 'Videos', 'open-tv');
  if (!existsSync(vPath))
    mkdirSync(vPath, { recursive: true });
  return vPath;
}

async function getCache() {
  if (!existsSync(cachePath))
    return {};
  let cacheJson = await readFile(cachePath, { encoding: "utf-8" });
  let cache = JSON.parse(cacheJson);
  let favs = [];
  if (existsSync(favsPath)) {
    let favsJson = await readFile(favsPath, { encoding: "utf-8" });
    favs = JSON.parse(favsJson);
  }
  return { cache: cache, favs: favs };
}

async function getSettings() {
  return settings;
}

async function fetchSettings() {
  if (existsSync(settingsPath)) {
    try {
      settings = JSON.parse(await readFile(settingsPath, { encoding: "utf-8" }));
    }
    catch (e) {
      console.error(`failed retrieving settings: ${e}`);
    }
  }
  applyDefaultSettings();
}

function applyDefaultSettings() {
  if (settings.useStreamCaching === undefined)
    settings.useStreamCaching = true;
}

async function saveToCache(data) {
  let json = JSON.stringify(data);
  if (!existsSync(appDataPath))
    await mkdir(appDataPath, { recursive: true });
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
    channel.type = URLIsNotLivestream(channel.url) ? movie : livestream;
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
  //Skip first line because it's always a dud line in m3u files
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
  //Skip first line because it's always a dud line in m3u files
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
  if (URLIsNotLivestream(url))
    command += " --save-position-on-quit";
  else
    command += ` --cache=${settings.useStreamCaching === true ? 'auto' : 'no'} --hls-bitrate=max --prefetch-playlist=yes --loop-playlist=inf`;
  if (record === true) {
    let recordingFilePath = join(settings?.recordingPath ?? defaultRecordingPath, getRecordingFileName());
    command += ` --stream-record="${recordingFilePath}"`
  }
  let child = await exec(command);
  mpvProcesses.push(child);
  console.log("Waiting for mpv start");
  await waitForProcessStart(child);
  console.log(`Playing channel from URL: ${url}`);
}

function URLIsNotLivestream(url) {
  return url.endsWith(".mp4") || url.endsWith(".mkv")
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
  //Due to finder not giving us the path env variable in MacOS, we hardcode it
  //@TODO find a better solution
  if (process.platform == 'darwin') {
    if (existsSync("/opt/homebrew/bin/mpv"))
      mpvPath = "/opt/homebrew/bin/mpv";
    else if (existsSync("/opt/local/mpv"))
      mpvPath = "/opt/local/mpv";
  }
  /**
   * On Windows some builds of OpenTV have MPV included
   * If we can find it in path, it's prioritized.
   */
  if (process.platform == "win32") {
    let mpvExists = await lookpath("mpv");
    if (!mpvExists) {
      mpvPath = join(__dirname, '..', 'libs', 'mpv.exe');
    }
  }
}