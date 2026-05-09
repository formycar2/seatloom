import React from 'react'
import ReactDOM from 'react-dom/client'
import './styles/globals.css'
import './app-v2/styles/tokens.css'
import AppV2 from './app-v2/AppV2'
import { TopErrorBoundary } from './TopErrorBoundary'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <TopErrorBoundary>
      <AppV2 />
    </TopErrorBoundary>
  </React.StrictMode>,
)
