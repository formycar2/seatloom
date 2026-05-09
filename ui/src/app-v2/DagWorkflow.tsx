/**
 * Professional DAG Workflow Renderer
 * Inspired by AntV X6 design patterns.
 */

import React, { useState, useMemo, useRef, useEffect } from 'react';
import { 
  CheckCircle2, 
  Clock3, 
  AlertCircle, 
  XCircle, 
  Lock, 
  RefreshCw,
  Info
} from 'lucide-react';
import {
  WorkNode,
  StageWorkflow,
  computeLayers,
  getDownstream,
  getUpstream,
  defaultStatusColor,
  WorkNodeStatus,
} from './dag-model';

// ─── Status Icon Mapping ───

const StatusIcon: React.FC<{ status: WorkNodeStatus; color: string; size?: number }> = ({ status, color, size = 14 }) => {
  switch (status) {
    case 'done': return <CheckCircle2 size={size} style={{ color }} />;
    case 'active': return <RefreshCw size={size} className="animate-spin-slow" style={{ color }} />;
    case 'blocked': return <AlertCircle size={size} style={{ color }} />;
    case 'failed': return <XCircle size={size} style={{ color }} />;
    case 'waiting': return <Clock3 size={size} style={{ color }} />;
    case 'revisited': return <RefreshCw size={size} style={{ color }} />;
    default: return <Lock size={size} style={{ color }} />;
  }
};

// ─── Modern Node Card ───

const DagNode: React.FC<{
  node: WorkNode;
  selected: boolean;
  onSelect: (id: string | null) => void;
  registerRef: (id: string, el: HTMLDivElement | null) => void;
  onMouseEnter?: (e: React.MouseEvent, node: WorkNode) => void;
}> = ({ node, selected, onSelect, registerRef, onMouseEnter }) => {
  const [hovered, setHovered] = useState(false);
  const color = node.accentColor || defaultStatusColor(node.status);
  
  const isActive = node.status === 'active';
  const isBlocked = node.status === 'blocked';

  return (
    <div
      ref={(el) => registerRef(node.id, el)}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(selected ? null : node.id);
      }}
      onMouseEnter={(e) => {
        setHovered(true);
        onMouseEnter?.(e, node);
      }}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: 180,
        background: 'var(--sl-surface)',
        borderRadius: 'var(--sl-radius-md)',
        border: `1px solid ${selected ? color : hovered ? 'var(--sl-border)' : 'var(--sl-border-light)'}`,
        boxShadow: selected ? `0 4px 12px ${color}25` : hovered ? 'var(--sl-shadow-sm)' : 'none',
        cursor: 'pointer',
        transition: 'all 200ms ease',
        position: 'relative',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        zIndex: 2, // Ensure nodes are above edges for clicks
      }}
    >
      <div style={{
        position: 'absolute', left: 0, top: 0, bottom: 0, width: 4,
        background: color,
        opacity: isActive || isBlocked || selected ? 1 : 0.4
      }} />

      <div style={{ padding: '8px 10px 6px 12px', display: 'flex', alignItems: 'center', gap: 8 }}>
        <StatusIcon status={node.status} color={color} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ 
            fontSize: 11, fontWeight: 700, color: 'var(--sl-text-primary)',
            overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap'
          }}>
            {node.owner || node.label}
          </div>
        </div>
      </div>

      <div style={{ padding: '0 12px 8px 12px' }}>
        <div style={{ 
          fontSize: 12, fontWeight: 500, color: 'var(--sl-text-secondary)',
          lineHeight: 1.3, display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical',
          overflow: 'hidden'
        }}>
          {node.label}
        </div>

        {node.progress && (
          <div style={{ marginTop: 6, display: 'flex', alignItems: 'center', gap: 6 }}>
            <div style={{ flex: 1, height: 3, borderRadius: 1.5, background: 'var(--sl-bg)', overflow: 'hidden' }}>
              <div style={{ width: `${(node.progress.done / node.progress.total) * 100}%`, height: '100%', background: color }} />
            </div>
            <span style={{ fontSize: 9, color: 'var(--sl-text-tertiary)', fontWeight: 600 }}>
              {Math.round((node.progress.done / node.progress.total) * 100)}%
            </span>
          </div>
        )}
      </div>
      
      {selected && (
        <div style={{ 
          position: 'absolute', inset: 0, background: `${color}05`, 
          pointerEvents: 'none', border: `1.5px solid ${color}` 
        }} />
      )}
    </div>
  );
};

// ─── Edge Renderer (Curved Bezier) ───

const CurvedEdge: React.FC<{
  start: { x: number; y: number };
  end: { x: number; y: number };
  color: string;
  active?: boolean;
}> = ({ start, end, color, active }) => {
  const dx = Math.abs(end.x - start.x);
  const cp1x = start.x + dx * 0.4;
  const cp2x = end.x - dx * 0.4;

  const path = `M ${start.x} ${start.y} C ${cp1x} ${start.y}, ${cp2x} ${end.y}, ${end.x} ${end.y}`;

  return (
    <g>
      <path
        d={path}
        fill="none"
        stroke={color}
        strokeWidth={active ? 2.5 : 1.5}
        strokeOpacity={active ? 0.9 : 0.25}
        style={{ transition: 'all 300ms ease' }}
      />
      <path
        d={`M ${end.x - 6} ${end.y - 4} L ${end.x} ${end.y} L ${end.x - 6} ${end.y + 4}`}
        fill="none"
        stroke={color}
        strokeWidth={active ? 2 : 1.5}
        strokeOpacity={active ? 0.9 : 0.25}
      />
    </g>
  );
};

// ─── Main Component ───

export const DagWorkflow: React.FC<{
  workflow: StageWorkflow;
  onNodeMouseEnter?: (e: React.MouseEvent, node: WorkNode) => void;
}> = ({ workflow, onNodeMouseEnter }) => {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const contentWrapperRef = useRef<HTMLDivElement>(null);
  const nodeRefs = useRef<Record<string, HTMLDivElement | null>>({});
  const [coords, setCoords] = useState<Record<string, { x: number; y: number, leftX: number }>>({});

  const layers = useMemo(() => computeLayers(workflow.nodes), [workflow.nodes]);

  const updateCoords = () => {
    if (!contentWrapperRef.current) return;
    const wrapperRect = contentWrapperRef.current.getBoundingClientRect();
    const newCoords: Record<string, { x: number; y: number, leftX: number }> = {};

    Object.entries(nodeRefs.current).forEach(([id, el]) => {
      if (el) {
        const rect = el.getBoundingClientRect();
        newCoords[id] = {
          x: rect.left - wrapperRect.left + rect.width, // Right connection point
          y: rect.top - wrapperRect.top + rect.height / 2, // Vertical center
          leftX: rect.left - wrapperRect.left, // Left connection point
        };
      }
    });
    setCoords(newCoords);
  };

  useEffect(() => {
    updateCoords();
    const timer = setTimeout(updateCoords, 150); // Delay for layout settling
    window.addEventListener('resize', updateCoords);
    return () => {
      clearTimeout(timer);
      window.removeEventListener('resize', updateCoords);
    };
  }, [layers, selectedId]);

  const edges = useMemo(() => {
    const e: { from: string; to: string; color: string; active: boolean }[] = [];
    workflow.nodes.forEach(node => {
      node.dependsOn.forEach(upId => {
        const upNode = workflow.nodes.find(n => n.id === upId);
        e.push({
          from: upId,
          to: node.id,
          color: upNode?.status === 'done' ? 'var(--sl-green)' : 'var(--sl-border)',
          active: selectedId === node.id || selectedId === upId,
        });
      });
    });
    return e;
  }, [workflow.nodes, selectedId]);

  return (
    <div style={{ position: 'relative' }}>
      <div 
        ref={scrollContainerRef}
        style={{
          overflowX: 'auto',
          width: '100%',
          paddingBottom: 10,
        }}
      >
        <div 
          ref={contentWrapperRef}
          style={{
            position: 'relative',
            display: 'inline-flex',
            alignItems: 'flex-start',
            gap: 60,
            padding: '20px 10px',
            minWidth: '100%',
            minHeight: 180,
          }}
          onClick={() => setSelectedId(null)}
        >
          {layers.map((layer, li) => (
            <div key={li} style={{ display: 'flex', flexDirection: 'column', gap: 12, justifyContent: 'center', height: '100%', minHeight: 140 }}>
              {layer.map(node => (
                <DagNode
                  key={node.id}
                  node={node}
                  selected={selectedId === node.id}
                  onSelect={setSelectedId}
                  onMouseEnter={onNodeMouseEnter}
                  registerRef={(id, el) => { nodeRefs.current[id] = el; }}
                />
              ))}
            </div>
          ))}

          <svg 
            style={{ 
              position: 'absolute', inset: 0, pointerEvents: 'none', 
              width: '100%', height: '100%', zIndex: 0 
            }}
          >
            {edges.map((edge, idx) => {
              const start = coords[edge.from];
              const end = coords[edge.to];
              if (!start || !end) return null;
              return (
                <CurvedEdge 
                  key={idx}
                  start={start}
                  end={{ x: end.leftX, y: end.y }}
                  color={edge.color}
                  active={edge.active}
                />
              );
            })}
          </svg>
        </div>
      </div>
    </div>
  );
};

export default DagWorkflow;
