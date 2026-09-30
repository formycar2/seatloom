// ProvenanceBanner — the "is this real?" marker.
//
// UM0 exists because the app could be opened and read as live when almost all
// of it was fixture data. The fix is not subtlety: every surface states what it
// is, in the same place, in the same shape, every time. Real is green and
// moving; sample is amber and says so in words.
//
// ProvenanceFrame wraps a view without modifying it, so the V1 views keep their
// own layout and this file stays the single place the wording lives.

import React from 'react';
import { AlertTriangle, ArrowRight, Radio } from 'lucide-react';
import { PROVENANCE_META, type Provenance } from '../lib/provenance';

interface BannerProps {
  provenance: Provenance;
  /** Jump to the live mirror. Shown only on sample/mock banners. */
  onNavigateToLive?: () => void;
}

export const ProvenanceBanner: React.FC<BannerProps> = ({ provenance, onNavigateToLive }) => {
  const meta = PROVENANCE_META[provenance];
  const isLive = provenance === 'live';
  const Icon = isLive ? Radio : AlertTriangle;

  return (
    <div
      role="status"
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        gap: 10,
        padding: '10px 16px',
        background: meta.bg,
        borderBottom: `1px solid ${meta.border}`,
        // Left keyline makes the state readable from the edge of vision even
        // when the viewport is scrolled to a dense table.
        boxShadow: `inset 3px 0 0 0 ${meta.border}`,
        flexShrink: 0,
      }}
    >
      <Icon size={15} style={{ color: meta.fg, marginTop: 1, flexShrink: 0 }} />

      <div style={{ minWidth: 0, flex: 1 }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            flexWrap: 'wrap',
            marginBottom: 2,
          }}
        >
          <span
            style={{
              fontSize: 10,
              fontWeight: 800,
              letterSpacing: '0.08em',
              padding: '1px 6px',
              borderRadius: 'var(--sl-radius-sm)',
              background: meta.fg,
              color: '#fff',
              flexShrink: 0,
            }}
          >
            {meta.badge}
          </span>
          <span style={{ fontSize: 12.5, fontWeight: 700, color: meta.fg }}>{meta.headline}</span>
        </div>
        <div style={{ fontSize: 11.5, lineHeight: 1.5, color: meta.fg, opacity: 0.85 }}>
          {meta.detail}
        </div>
      </div>

      {!isLive && onNavigateToLive && (
        <button
          onClick={onNavigateToLive}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 4,
            padding: '4px 10px',
            fontSize: 11.5,
            fontWeight: 700,
            color: '#fff',
            background: 'var(--sl-green)',
            border: 'none',
            borderRadius: 'var(--sl-radius-sm)',
            cursor: 'pointer',
            flexShrink: 0,
            whiteSpace: 'nowrap',
          }}
        >
          查看实时席位
          <ArrowRight size={13} />
        </button>
      )}
    </div>
  );
};

interface FrameProps {
  provenance: Provenance;
  children: React.ReactNode;
  onNavigateToLive?: () => void;
}

/**
 * Banner above, untouched view below. Used from App.renderMainView so no
 * individual V1 view needs editing to gain a provenance marker.
 */
export const ProvenanceFrame: React.FC<FrameProps> = ({
  provenance,
  children,
  onNavigateToLive,
}) => (
  <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
    <ProvenanceBanner provenance={provenance} onNavigateToLive={onNavigateToLive} />
    <div style={{ display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
      {children}
    </div>
  </div>
);

/**
 * Nav-rail marker so provenance is legible BEFORE a tab is opened — "at a
 * glance" has to include the glance you take while deciding where to click.
 */
export const ProvenanceDot: React.FC<{ provenance: Provenance; title?: string }> = ({
  provenance,
  title,
}) => {
  const meta = PROVENANCE_META[provenance];
  const isLive = provenance === 'live';
  return (
    <span
      title={title ?? meta.headline}
      style={{
        width: 6,
        height: 6,
        borderRadius: '50%',
        background: meta.fg,
        flexShrink: 0,
        boxShadow: isLive ? `0 0 0 2px ${meta.bg}` : 'none',
      }}
    />
  );
};
