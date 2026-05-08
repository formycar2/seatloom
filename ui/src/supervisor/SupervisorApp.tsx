// SupervisorApp — root component for the detached supervisor window.
// Mounts V2's SupervisorPanel (the canonical IM, unchanged) full-viewport,
// plus a re-embed button that closes this window.
//
// CSS: imports both globals.css (Tailwind + design tokens used by V1
// components) and the v2 tokens.css (CSS vars used by SupervisorPanel).

import React, { useEffect } from 'react';
import { SupervisorPanel } from '../app-v2/panel/SupervisorPanel';
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
      <SupervisorPanel onClose={handleReembed} />
      <button
        onClick={handleReembed}
        title="Re-embed in main window"
        style={{
          position: 'fixed', top: 12, right: 18, zIndex: 10000,
          padding: '4px 12px', fontSize: 12, fontWeight: 600,
          background: 'var(--sl-surface)', color: 'var(--sl-text-primary)',
          border: '1px solid var(--sl-border)', borderRadius: 6,
          cursor: 'pointer', boxShadow: '0 2px 6px rgba(0,0,0,0.08)',
        }}
      >↙ Re-embed</button>
    </div>
  );
};
