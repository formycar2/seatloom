import React from 'react';
import ReactDOM from 'react-dom/client';
import { SupervisorApp } from './SupervisorApp';
import '../styles/globals.css';
import '../app-v2/styles/tokens.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <SupervisorApp />
  </React.StrictMode>,
);
