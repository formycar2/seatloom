// Top-level error boundary catches any render-time error in the V1 shell or
// its children (post-hydration shape mismatches, etc.) so the app shows a
// readable error card instead of a white screen.

import React from 'react';

interface State { hasError: boolean; message: string; stack: string }

export class TopErrorBoundary extends React.Component<
  { children: React.ReactNode },
  State
> {
  state: State = { hasError: false, message: '', stack: '' };

  static getDerivedStateFromError(err: unknown): State {
    return {
      hasError: true,
      message: err instanceof Error ? err.message : String(err),
      stack: err instanceof Error ? (err.stack ?? '') : '',
    };
  }

  componentDidCatch(err: unknown, info: unknown) {
    console.error('[TopErrorBoundary] App crashed:', err, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{
          padding: 24, fontFamily: 'ui-monospace, "SF Mono", monospace',
          fontSize: 12, color: '#1A1A2E', background: '#F8F9FA',
          height: '100vh', overflow: 'auto',
        }}>
          <h1 style={{ fontSize: 18, fontWeight: 700, color: '#EF4444', marginBottom: 12 }}>
            SeatLoom crashed during render
          </h1>
          <div style={{ marginBottom: 16 }}>
            <strong>Message:</strong> {this.state.message}
          </div>
          {this.state.stack && (
            <details open style={{ marginBottom: 16 }}>
              <summary style={{ cursor: 'pointer', fontWeight: 600, marginBottom: 8 }}>
                Stack
              </summary>
              <pre style={{ whiteSpace: 'pre-wrap', fontSize: 11, lineHeight: 1.5, background: '#FFFFFF', padding: 12, borderRadius: 4 }}>
                {this.state.stack}
              </pre>
            </details>
          )}
          <button
            onClick={() => location.reload()}
            style={{
              padding: '6px 14px', fontSize: 13, fontWeight: 600,
              background: '#3B82F6', color: 'white',
              border: 'none', borderRadius: 4, cursor: 'pointer',
              marginRight: 8,
            }}
          >Reload</button>
          <button
            onClick={() => this.setState({ hasError: false, message: '', stack: '' })}
            style={{
              padding: '6px 14px', fontSize: 13,
              background: 'transparent', color: '#1A1A2E',
              border: '1px solid #E5E7EB', borderRadius: 4, cursor: 'pointer',
            }}
          >Dismiss</button>
        </div>
      );
    }
    return this.props.children;
  }
}
