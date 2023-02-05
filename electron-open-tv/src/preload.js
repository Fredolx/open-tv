const {ipcRenderer, contextBridge} = require("electron");

contextBridge.exposeInMainWorld('electronAPI', {
    selectFile: () => ipcRenderer.invoke('selectFile'),
    getCache: () => ipcRenderer.invoke('getCache'),
    playChannel: (url) => ipcRenderer.invoke('playChannel', url),
    deleteCache: () => ipcRenderer.invoke('deleteCache'),
    saveFavs: (favs) => ipcRenderer.invoke('saveFavs', favs)
});