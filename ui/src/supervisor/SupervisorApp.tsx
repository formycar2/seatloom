// SupervisorApp — root component for the detached supervisor window.
//
// Hydrates the useDataStore from the backend just like the main window does
// (each webview process is independent, so it runs its own hydration).
// Then renders <SupervisorIM embedded={false} /> full-viewport.

import React, { useEffect } from 'react';
import { SupervisorIM } from './SupervisorIM';
import { useDataStore } from '../stores/useDataStore';
import { api, isTauri } from '../lib/api';

export const SupervisorApp: React.FC = () => {
  const hydrateFromBackend = useDataStore((s) => s.hydrateFromBackend);

  useEffect(() => {
    hydrateFromBackend();
  }, [hydrateFromBackend]);

  const handleReembed = async () => {
    if (!isTauri()) return;
    try {
      await api.closeSupervisorWindow();
    } catch (e) {
      console.error('closeSupervisorWindow failed', e);
    }
  };

  return (
    <div style={{ width: '100vw', height: '100vh', background: 'var(--sl-bg)' }}>
      <SupervisorIM embedded={false} onReembed={handleReembed} />
    </div>
  );
};
