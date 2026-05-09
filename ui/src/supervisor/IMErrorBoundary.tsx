// Error boundary used to wrap the embedded Supervisor IM panel so a runtime
// error inside the V2 SupervisorPanel does not blank the entire V1 main
// window. Falls back to a small inline notice with the error message and a
// reload button.

import React from 'react';

interface State { hasError: boolean; message: string }

export class IMErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  state: State = { hasError: false, message: '' };

  static getDerivedStateFromError(err: unknown): State {
    return { hasError: true, message: err instanceof Error ? err.message : String(err) };
  }

  componentDidCatch(err: unknown, info: unknown) {
    console.error('[IMErrorBoundary] Supervisor IM crashed:', err, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          position: 'fixed', right: 20, bottom: 20, width: 360,
          padding: 16, background: 'var(--sl-surface)',
          border: '1px solid var(--sl-red, #ef4444)', borderRadius: 8,
          boxShadow: '0 8px 24px rgba(0,0,0,0.18)', fontSize: 12,
          color: 'var(--sl-text-primary)', zIndex: 10000,
        }}>
          <div style={{ fontWeight: 700, marginBottom: 6 }}>Supervisor IM crashed</div>
          <pre style={{ whiteSpace: 'pre-wrap', margin: 0, fontSize: 11, color: 'var(--sl-text-secondary)' }}>
            {this.state.message}
          </pre>
          <button
            onClick={() => this.setState({ hasError: false, message: '' })}
            style={{
              marginTop: 8, padding: '4px 10px', fontSize: 12,
              background: 'var(--sl-brand)', color: 'white',
              border: 'none', borderRadius: 4, cursor: 'pointer',
            }}
          >Retry</button>
        </div>
      );
    }
    return this.props.children;
  }
}
