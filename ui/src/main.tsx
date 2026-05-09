import React from 'react'
import ReactDOM from 'react-dom/client'
import './styles/globals.css'
import './app-v2/styles/tokens.css'
import App from './App'
import { TopErrorBoundary } from './TopErrorBoundary'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <TopErrorBoundary>
      <App />
    </TopErrorBoundary>
  </React.StrictMode>,
)
