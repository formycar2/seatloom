/**
 * Avatar.tsx
 * 通用小组件：SeatLoomLogo（Logo SVG）、StatusDot（带颜色点和标签的状态指示器）、Avatar（单字母圆形头像，带在线状态）
 */

import React from 'react';

export const SeatLoomLogo: React.FC<{ size?: number }> = ({ size = 22 }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
    <rect width="24" height="24" rx="6" fill="var(--sl-brand)" />
    <path d="M7 8.5C7 8.5 9.5 6 12 6C14.5 6 17 8.5 17 8.5" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
    <path d="M7 12C7 12 9.5 14.5 12 14.5C14.5 14.5 17 12 17 12" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
    <path d="M7 15.5C7 15.5 9.5 18 12 18C14.5 18 17 15.5 17 15.5" stroke="white" strokeWidth="1.8" strokeLinecap="round" />
  </svg>
);

export const StatusDot: React.FC<{ color: string; label: string; value?: number | string }> = ({ color, label, value }) => (
  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
    <div style={{ width: 8, height: 8, borderRadius: '50%', backgroundColor: color, boxShadow: `0 0 0 2px ${color}20` }} />
    <span style={{ fontSize: 12, color: 'var(--sl-text-secondary)', fontWeight: 500 }}>{label}</span>
    {value !== undefined && <span style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)' }}>{value}</span>}
  </div>
);

export const Avatar: React.FC<{ char: string; color: string; size?: number; online?: boolean }> = ({ char, color, size = 36, online }) => (
  <div style={{ position: 'relative', width: size, height: size, flexShrink: 0 }}>
    <div style={{
      width: size, height: size, borderRadius: size * 0.28,
      background: `${color}18`, color: color,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      fontSize: size * 0.38, fontWeight: 700,
      border: `1.5px solid ${color}40`,
    }}>{char}</div>
    {online !== undefined && (
      <div style={{
        position: 'absolute', bottom: -1, right: -1,
        width: 10, height: 10, borderRadius: '50%',
        background: online ? 'var(--sl-green)' : 'var(--sl-border)',
        border: '2px solid var(--sl-surface)',
      }} />
    )}
  </div>
);
