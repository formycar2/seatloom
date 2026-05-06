/**
 * MessageBubble.tsx
 * 消息气泡：渲染单条聊天消息，支持 text / delivery / verification / gate / blocker / progress / alert / info 等卡片类型
 */

import React, { useState } from 'react';
import { ChatMessage } from '../types';

export const TYPE_BADGES: Record<string, { label: string; color: string; bg: string }> = {
  delivery: { label: '交付', color: 'var(--sl-blue)', bg: 'var(--sl-blue-subtle)' },
  verification: { label: '验证', color: 'var(--sl-teal)', bg: 'var(--sl-teal-subtle)' },
  gate: { label: '审批', color: 'var(--sl-green)', bg: 'var(--sl-green-subtle)' },
  blocker: { label: '阻塞', color: 'var(--sl-red)', bg: 'var(--sl-red-subtle)' },
  progress: { label: '进度', color: 'var(--sl-amber)', bg: 'var(--sl-amber-subtle)' },
  alert: { label: '告警', color: 'var(--sl-red)', bg: 'var(--sl-red-subtle)' },
  info: { label: '摘要', color: 'var(--sl-brand)', bg: 'var(--sl-brand-subtle)' },
};

export const MessageBubble: React.FC<{ msg: ChatMessage }> = ({ msg }) => {
  const [showDetail, setShowDetail] = useState(false);
  const [hovered, setHovered] = useState(false);

  // Plain text from user
  if (msg.type === 'text') {
    return (
      <div style={{
        display: 'flex', flexDirection: 'column',
        alignItems: msg.from === 'user' ? 'flex-end' : 'flex-start',
        alignSelf: msg.from === 'user' ? 'flex-end' : 'flex-start',
        maxWidth: '78%',
      }}>
        <div style={{
          padding: '8px 12px',
          borderRadius: msg.from === 'user' ? '12px 12px 2px 12px' : '12px 12px 12px 2px',
          background: msg.from === 'user' ? 'var(--sl-brand)' : 'var(--sl-surface-hover)',
          color: msg.from === 'user' ? 'white' : 'var(--sl-text-primary)',
          fontSize: 13, lineHeight: 1.5,
          boxShadow: 'var(--sl-shadow-sm)',
        }}>
          {msg.content}
        </div>
        <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 2, padding: '0 4px' }}>{msg.time}</span>
      </div>
    );
  }

  // Structured card
  const badge = TYPE_BADGES[msg.type] || TYPE_BADGES.info;
  const card = msg.card!;

  return (
    <div style={{
      alignSelf: msg.from === 'user' ? 'flex-end' : 'flex-start',
      maxWidth: '85%', position: 'relative',
    }}>
      <div
        onClick={() => card.detail && setShowDetail(true)}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        style={{
          padding: '10px 14px',
          borderRadius: 'var(--sl-radius-lg)',
          border: `1px solid ${hovered ? badge.color + '60' : 'var(--sl-border)'}`,
          background: 'var(--sl-surface)',
          boxShadow: hovered ? 'var(--sl-shadow-md)' : 'var(--sl-shadow-sm)',
          cursor: card.detail ? 'pointer' : 'default',
          transition: 'all 150ms ease',
          minWidth: 220,
        }}
      >
        {/* Type badge + title */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
          <span style={{
            fontSize: 10, fontWeight: 600, padding: '2px 6px',
            borderRadius: 'var(--sl-radius-full)',
            background: badge.bg, color: badge.color,
          }}>{badge.label}</span>
          <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{card.title}</span>
        </div>

        {/* Status line */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: card.fields?.length ? 8 : 0 }}>
          {card.statusColor && <div style={{ width: 6, height: 6, borderRadius: '50%', background: card.statusColor }} />}
          <span style={{ fontSize: 12, color: 'var(--sl-text-secondary)' }}>{card.status}</span>
        </div>

        {/* Fields (key-value pairs) */}
        {card.fields && card.fields.length > 0 && (
          <div style={{
            display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '3px 12px',
            fontSize: 12, marginBottom: card.actions?.length ? 8 : 0,
            padding: '6px 0',
            borderTop: '1px solid var(--sl-divider)',
          }}>
            {card.fields.map(f => (
              <React.Fragment key={f.label}>
                <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 500 }}>{f.label}</span>
                <span style={{ color: 'var(--sl-text-primary)' }}>{f.value}</span>
              </React.Fragment>
            ))}
          </div>
        )}

        {/* Action buttons */}
        {card.actions && card.actions.length > 0 && (
          <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', paddingTop: 4 }}>
            {card.actions.map(a => (
              <button key={a} onClick={e => e.stopPropagation()} style={{
                padding: '3px 10px', fontSize: 11, fontWeight: 500,
                color: 'var(--sl-brand)', background: 'var(--sl-brand-subtle)',
                border: '1px solid transparent', borderRadius: 'var(--sl-radius-full)',
                cursor: 'pointer', transition: 'all 120ms ease',
              }}
                onMouseEnter={e => { e.currentTarget.style.borderColor = 'var(--sl-brand)'; }}
                onMouseLeave={e => { e.currentTarget.style.borderColor = 'transparent'; }}
              >{a}</button>
            ))}
          </div>
        )}

        {/* "Click for detail" hint */}
        {card.detail && hovered && (
          <div style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 4, textAlign: 'right' }}>
            点击查看详情
          </div>
        )}
      </div>
      <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', marginTop: 2, padding: '0 4px', display: 'block' }}>{msg.time}</span>

      {/* Detail popup */}
      {showDetail && card.detail && (
        <>
          <div onClick={() => setShowDetail(false)} style={{ position: 'fixed', inset: 0, zIndex: 10000, background: 'rgba(0,0,0,0.2)' }} />
          <div style={{
            position: 'fixed', top: '50%', left: '50%', transform: 'translate(-50%,-50%)',
            zIndex: 10001, width: 420, background: 'var(--sl-surface)',
            border: '1px solid var(--sl-border)', borderRadius: 'var(--sl-radius-xl)',
            boxShadow: 'var(--sl-shadow-overlay)', padding: '20px',
          }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ fontSize: 10, fontWeight: 600, padding: '2px 6px', borderRadius: 'var(--sl-radius-full)', background: badge.bg, color: badge.color }}>{badge.label}</span>
                <span style={{ fontSize: 15, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{card.title}</span>
              </div>
              <button onClick={() => setShowDetail(false)} style={{ border: 'none', background: 'transparent', fontSize: 16, color: 'var(--sl-text-tertiary)', cursor: 'pointer' }}>✕</button>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12 }}>
              {card.statusColor && <div style={{ width: 8, height: 8, borderRadius: '50%', background: card.statusColor }} />}
              <span style={{ fontSize: 13, fontWeight: 500, color: 'var(--sl-text-secondary)' }}>{card.status}</span>
            </div>
            {card.fields && (
              <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '6px 16px', fontSize: 13, marginBottom: 16, padding: '12px', background: 'var(--sl-bg)', borderRadius: 'var(--sl-radius-md)' }}>
                {card.fields.map(f => (
                  <React.Fragment key={f.label}>
                    <span style={{ color: 'var(--sl-text-tertiary)', fontWeight: 500 }}>{f.label}</span>
                    <span style={{ color: 'var(--sl-text-primary)' }}>{f.value}</span>
                  </React.Fragment>
                ))}
              </div>
            )}
            <p style={{ fontSize: 13, lineHeight: 1.7, color: 'var(--sl-text-secondary)' }}>{card.detail}</p>
          </div>
        </>
      )}
    </div>
  );
};
