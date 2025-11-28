import { useState, useEffect } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/electron-vite.animate.svg'
import './App.css'

function App() {
  const [count, setCount] = useState(0)
  const [lspOutput, setLspOutput] = useState('')

  useEffect(() => {
    // @ts-ignore
    const removeListener = window.ipcRenderer.on('lsp-output', (_event, data) => {
      const text = new TextDecoder().decode(data)
      console.log('LSP Output:', text)
      setLspOutput(prev => prev + text)
    })

    const initRequest = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        processId: null,
        rootUri: null,
        capabilities: {}
      }
    })

    const message = `Content-Length: ${initRequest.length}\r\n\r\n${initRequest}`
    console.log('Sending LSP init:', message)
    // @ts-ignore
    window.ipcRenderer.send('lsp-input', message)

    return () => {
      (removeListener as any)()
    }
  }, [])

  return (
    <>
      <div>
        <a href="https://electron-vite.github.io" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React + LSP</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <div style={{ textAlign: 'left', marginTop: '20px', background: '#333', padding: '10px', borderRadius: '4px' }}>
          <h3>LSP Output:</h3>
          <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>{lspOutput}</pre>
        </div>
      </div>
    </>
  )
}

export default App
