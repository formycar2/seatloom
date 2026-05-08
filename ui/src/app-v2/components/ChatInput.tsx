/**
 * ChatInput.tsx
 * 消息输入框：带 routing governance 逻辑（向 worker/verifier 直发时弹出路由守卫对话框，建议通过 PO 中转）
 */

import React, { useState } from 'react';
import { ChatContact } from '../types';

export const ChatInput: React.FC<{
  contact: ChatContact;
  allContacts: ChatContact[];
  input: string;
  onInputChange: (v: string) => void;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onEscape: () => void;
  onRouteViaPO: (poId: string) => void;
  onSend?: (text: string) => void;
}> = ({ contact, allContacts, input, onInputChange, inputRef, onEscape, onRouteViaPO, onSend }) => {
  const [showRoutingGuard, setShowRoutingGuard] = useState(false);

  const isDirectWorker = contact.type === 'seat' && (contact.seatType === 'worker' || contact.seatType === 'verifier');
  const po = isDirectWorker
    ? allContacts.find(c => c.type === 'seat' && c.seatType === 'po' && c.projectId === contact.projectId)
    : null;

  const handleSend = () => {
    const text = input.trim();
    if (!text) return;
    if (isDirectWorker && !showRoutingGuard) {
      setShowRoutingGuard(true);
      return;
    }
    // Direct send (after guard confirmation or non-worker contact).
    setShowRoutingGuard(false);
    onSend?.(text);
    onInputChange('');
  };

  const handleRouteViaPO = () => {
    if (po) {
      setShowRoutingGuard(false);
      onRouteViaPO(po.id);
    }
  };

  return (
    <div style={{ borderTop: '1px solid var(--sl-divider)', flexShrink: 0 }}>
      {/* Routing guard banner */}
      {showRoutingGuard && po && (
        <div style={{
          padding: '10px 16px',
          background: 'var(--sl-amber-subtle)',
          borderBottom: '1px solid var(--sl-amber)',
          display: 'flex', flexDirection: 'column', gap: 8,
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ fontSize: 13 }}>⚡</span>
            <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)' }}>
              直接消息将绕过 {po.name} (PO)
            </span>
          </div>
          <div style={{ fontSize: 11, color: 'var(--sl-text-secondary)', lineHeight: 1.5 }}>
            通过 PO 转发的消息 token 效率更高——PO 会打包为精确的任务包。直接发送将自动通知 {po.name}。
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={handleRouteViaPO}
              style={{
                padding: '5px 14px', fontSize: 12, fontWeight: 600,
                background: 'var(--sl-brand)', color: 'white',
                border: 'none', borderRadius: 'var(--sl-radius-md)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
              onMouseEnter={e => { e.currentTarget.style.background = 'var(--sl-brand-hover)'; }}
              onMouseLeave={e => { e.currentTarget.style.background = 'var(--sl-brand)'; }}
            >
              通过 {po.name} 转发 (推荐)
            </button>
            <button
              onClick={() => { setShowRoutingGuard(false); onInputChange(''); }}
              style={{
                padding: '5px 14px', fontSize: 12, fontWeight: 500,
                background: 'transparent', color: 'var(--sl-text-secondary)',
                border: '1px solid var(--sl-border)', borderRadius: 'var(--sl-radius-md)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
            >
              仍然直接发送
            </button>
            <button
              onClick={() => setShowRoutingGuard(false)}
              style={{
                padding: '5px 10px', fontSize: 12,
                background: 'transparent', color: 'var(--sl-text-tertiary)',
                border: 'none', cursor: 'pointer',
              }}
            >
              取消
            </button>
          </div>
        </div>
      )}

      {/* Input area */}
      <div style={{ padding: '10px 16px 12px' }}>
        {/* Direct worker hint (subtle, always visible) */}
        {isDirectWorker && po && !showRoutingGuard && (
          <div style={{
            display: 'flex', alignItems: 'center', gap: 6,
            marginBottom: 6, fontSize: 11, color: 'var(--sl-text-tertiary)',
          }}>
            <span>⚡</span>
            <span>直接消息 · 将通知 {po.name}</span>
            <span style={{ color: 'var(--sl-border)' }}>|</span>
            <button
              onClick={() => po && onRouteViaPO(po.id)}
              style={{
                background: 'transparent', border: 'none', cursor: 'pointer',
                fontSize: 11, color: 'var(--sl-brand)', fontWeight: 500,
                padding: 0,
              }}
            >
              通过 {po.name} 转发 →
            </button>
          </div>
        )}

        <div style={{
          display: 'flex', alignItems: 'center', gap: 8,
          background: 'var(--sl-bg)',
          border: `1.5px solid ${isDirectWorker ? 'var(--sl-amber)' : 'var(--sl-border)'}`,
          borderRadius: 'var(--sl-radius-lg)', padding: '8px 12px',
          transition: 'border-color 120ms ease, box-shadow 120ms ease',
        }}
          onFocus={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; e.currentTarget.style.boxShadow = '0 0 0 3px var(--sl-brand-subtle)'; }}
          onBlur={e => { e.currentTarget.style.borderColor = isDirectWorker ? 'var(--sl-amber)' : 'var(--sl-border)'; e.currentTarget.style.boxShadow = 'none'; }}
        >
          <input
            ref={inputRef} type="text" value={input}
            onChange={e => onInputChange(e.target.value)}
            placeholder={`发消息给 ${contact.name}...`}
            style={{ flex: 1, border: 'none', outline: 'none', fontSize: 13, color: 'var(--sl-text-primary)', background: 'transparent' }}
            onKeyDown={e => {
              if (e.key === 'Escape') onEscape();
              if (e.key === 'Enter' && input.trim()) handleSend();
            }}
          />
          <button
            onClick={handleSend}
            style={{
              padding: '4px 12px', fontSize: 12, fontWeight: 600,
              color: input.trim() ? 'white' : 'var(--sl-text-tertiary)',
              background: input.trim() ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
              border: 'none', borderRadius: 'var(--sl-radius-md)',
              cursor: input.trim() ? 'pointer' : 'default', transition: 'all 120ms ease',
            }}
          >发送</button>
        </div>

        <div style={{ display: 'flex', gap: 8, marginTop: 6 }}>
          {['📎 附件', '📋 任务包', '🔍 证据'].map(a => (
            <button key={a} style={{
              fontSize: 11, color: 'var(--sl-text-tertiary)', background: 'transparent',
              border: 'none', cursor: 'pointer', padding: '2px 0', transition: 'color 120ms ease',
            }}
              onMouseEnter={e => { e.currentTarget.style.color = 'var(--sl-brand)'; }}
              onMouseLeave={e => { e.currentTarget.style.color = 'var(--sl-text-tertiary)'; }}
            >{a}</button>
          ))}
        </div>
      </div>
    </div>
  );
};
