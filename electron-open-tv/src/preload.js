const {ipcRenderer, contextBridge} = require("electron");

contextBridge.exposeInMainWorld('electronAPI', {
    selectFile: (name) => ipcRenderer.invoke('selectFile', name),
    getCache: () => ipcRenderer.invoke('getCache'),
    playChannel: (url, record) => ipcRenderer.invoke('playChannel', url, record),
    deleteAllCache: () => ipcRenderer.invoke('deleteAllCache'),
    saveFavs: (name, favs) => ipcRenderer.invoke("saveFavs", name, favs),
    downloadM3U: (name, url) => ipcRenderer.invoke('downloadM3U', name, url),
    selectFolder: () => ipcRenderer.invoke('selectFolder'),
    updateSettings: (settings) => ipcRenderer.invoke('updateSettings', settings),
    getSettings: () => ipcRenderer.invoke('getSettings'),
    getXtream: (name, xtream) => ipcRenderer.invoke('getXtream', name, xtream),
    getEpisodes: (series_data) => ipcRenderer.invoke('getEpisodes', series_data),
    deleteCache: (name) => ipcRenderer.invoke('deleteCache', name)
});