import {ipcRenderer, contextBridge} from "electron"

contextBridge.exposeInMainWorld('electronAPI', {
    selectFile: () => ipcRenderer.invoke('selectFile'),
    getCache: () => ipcRenderer.invoke('getCache'),
    playChannel: (url: string) => ipcRenderer.invoke('playChannel', url),
    deleteCache: () => ipcRenderer.invoke('deleteCache')
});
