"use strict";
const electron = require("electron");
electron.contextBridge.exposeInMainWorld("electronAPI", {
  selectFile: () => electron.ipcRenderer.invoke("selectFile"),
  getCache: () => electron.ipcRenderer.invoke("getCache")
});
//# sourceMappingURL=index.js.map
