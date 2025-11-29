import { ipcRenderer, contextBridge } from 'electron'

// --------- Expose some API to the Renderer process ---------
contextBridge.exposeInMainWorld('ipcRenderer', {
  on(...args: Parameters<typeof ipcRenderer.on>) {
    const [channel, listener] = args
    const subscription = (event: any, ...args: any[]) => listener(event, ...args)
    ipcRenderer.on(channel, subscription)
    return () => {
      ipcRenderer.removeListener(channel, subscription)
    }
  },
  off(...args: Parameters<typeof ipcRenderer.off>) {
    const [channel, ...omit] = args
    return ipcRenderer.off(channel, ...omit)
  },
  send(...args: Parameters<typeof ipcRenderer.send>) {
    const [channel, ...omit] = args
    return ipcRenderer.send(channel, ...omit)
  },
  invoke(...args: Parameters<typeof ipcRenderer.invoke>) {
    const [channel, ...omit] = args
    return ipcRenderer.invoke(channel, ...omit)
  },
  fileOpen: () => ipcRenderer.invoke('file-open'),
  fileSave: (filePath: string, content: string) => ipcRenderer.invoke('file-save', filePath, content),
  fileReadDir: (dirPath: string) => ipcRenderer.invoke('file-read-dir', dirPath),
  fileRead: (filePath: string) => ipcRenderer.invoke('file-read', filePath),
  folderOpen: () => ipcRenderer.invoke('folder-open'),
  getBenchmarkFile: () => ipcRenderer.invoke('get-benchmark-file'),
})
