/**
 * ContactRow.tsx
 * 联系人列表行：渲染单个联系人条目（头像、名字、角色、未读数、最新消息预览）
 */

import React from 'react';
import { ChatContact } from '../types';
import { Avatar } from './Avatar';

export const ContactRow: React.FC<{ contact: ChatContact; active: boolean; onClick: () => void }> = ({ contact, active, onClick }) => (
  <button onClick={onClick} style={{
    width: '100%', display: 'flex', alignItems: 'center', gap: 8,
    padding: '8px 12px', border: 'none', textAlign: 'left',
    background: active ? 'var(--sl-surface)' : 'transparent',
    cursor: 'pointer', transition: 'background 100ms ease',
    borderLeft: active ? '2px solid var(--sl-brand)' : '2px solid transparent',
  }}
    onMouseEnter={e => { if (!active) e.currentTarget.style.background = 'var(--sl-surface-hover)'; }}
    onMouseLeave={e => { if (!active) e.currentTarget.style.background = 'transparent'; }}
  >
    <Avatar char={contact.avatar} color={contact.color} size={32} online={contact.type === 'seat' ? contact.online : undefined} />
    <div style={{ flex: 1, minWidth: 0 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: 12, fontWeight: active ? 600 : 500, color: 'var(--sl-text-primary)', truncate: true } as any}>{contact.name}</span>
        {contact.lastTime && <span style={{ fontSize: 10, color: 'var(--sl-text-tertiary)', flexShrink: 0 }}>{contact.lastTime}</span>}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginTop: 1 }}>
        <span style={{
          fontSize: 11, color: 'var(--sl-text-tertiary)',
          overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: 120,
        }}>{contact.lastMessage || contact.role || ''}</span>
        {contact.unread > 0 && (
          <span style={{
            minWidth: 16, height: 16, borderRadius: 8,
            background: 'var(--sl-brand)', color: 'white',
            fontSize: 10, fontWeight: 600,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            padding: '0 4px', flexShrink: 0,
          }}>{contact.unread}</span>
        )}
      </div>
    </div>
  </button>
);
