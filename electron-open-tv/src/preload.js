const {ipcRenderer, contextBridge} = require("electron");

contextBridge.exposeInMainWorld('electronAPI', {
    selectFile: () => ipcRenderer.invoke('selectFile'),
    getCache: () => ipcRenderer.invoke('getCache'),
    playChannel: (url, record) => ipcRenderer.invoke('playChannel', url, record),
    deleteCache: () => ipcRenderer.invoke('deleteCache'),
    saveFavs: (favs) => ipcRenderer.invoke('saveFavs', favs),
    downloadM3U: (url) => ipcRenderer.invoke('downloadM3U', url),
    selectFolder: () => ipcRenderer.invoke('selectFolder'),
    updateSettings: (settings) => ipcRenderer.invoke('updateSettings', settings),
    getSettings: () => ipcRenderer.invoke('getSettings')
});