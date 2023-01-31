import {ipcRenderer, contextBridge} from "electron"

contextBridge.exposeInMainWorld('electronAPI', {
    selectFile: () => ipcRenderer.invoke('selectFile'),
    getCache: () => ipcRenderer.invoke('getCache')
});
