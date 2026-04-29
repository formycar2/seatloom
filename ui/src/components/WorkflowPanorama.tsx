import React from 'react';
import { useDataStore } from '../stores/useDataStore';
import { SeatRole, SeatStatus } from '../types';

interface SeatNode {
  id: string;
  name: string;
  role: SeatRole;
  activeCount: number;
  blocked: boolean;
  status: SeatStatus;
}

const WorkflowPanorama: React.FC = () => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentData) return null;

  const seatNodes: SeatNode[] = currentData.seats.map((seat) => {
    const ownedWIs = currentData.workItems.filter(
      (wi) => wi.owner_seat_id === seat.id && wi.status !== 'Done' && wi.status !== 'Verified'
    );
    const hasBlocked = ownedWIs.some((wi) => wi.status === 'Blocked') ||
      currentData.sessions.some((s) => s.seat_id === seat.id && s.status === 'InputRequired');

    return {
      id: seat.id,
      name: seat.name.charAt(0).toUpperCase() + seat.name.slice(1),
      role: seat.role,
      activeCount: ownedWIs.length,
      blocked: hasBlocked,
      status: seat.status,
    };
  });

  const activeNodes = seatNodes.filter((n) => n.status === 'Active');

  return (
    <div className="sl-card p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-[var(--sl-ink)]">工作流全景</h3>
        <span className="text-caption text-[var(--sl-ink-muted)]">席位流转</span>
      </div>

      <div className="flex items-center justify-center gap-2 py-3 overflow-x-auto">
        {activeNodes.map((node, i) => (
          <React.Fragment key={node.id}>
            {/* Seat node */}
            <div
              className={`
                flex flex-col items-center gap-1.5 px-4 py-3 rounded-lg border transition-all min-w-[80px]
                ${node.blocked
                  ? 'border-[var(--sl-error)] bg-[var(--sl-error-light)]'
                  : 'border-[var(--sl-border)] bg-canvas hover:border-primary/40'
                }
              `}
            >
              <div
                className={`
                  w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold
                  ${node.blocked
                    ? 'bg-[var(--sl-error)] text-white'
                    : 'bg-primary/10 text-primary'
                  }
                `}
              >
                {node.name.charAt(0)}
              </div>
              <span className="text-xs font-semibold text-[var(--sl-ink)] truncate max-w-[72px]">{node.name}</span>
              <span className={`text-[10px] font-medium ${node.blocked ? 'text-[var(--sl-error)]' : 'text-[var(--sl-ink-secondary)]'}`}>
                {node.blocked ? '阻塞' : `${node.activeCount} 项`}
              </span>
            </div>

            {/* Connector arrow */}
            {i < activeNodes.length - 1 && (
              <div className="flex items-center text-[var(--sl-ink-muted)] shrink-0">
                <div className="w-6 h-px bg-[var(--sl-border)]" />
                <svg width="8" height="12" viewBox="0 0 8 12" fill="none" className="shrink-0">
                  <path d="M1 1L6 6L1 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
            )}
          </React.Fragment>
        ))}

        {/* Gate node */}
        <div className="flex items-center text-[var(--sl-ink-muted)] shrink-0">
          <div className="w-6 h-px bg-[var(--sl-border)]" />
          <svg width="8" height="12" viewBox="0 0 8 12" fill="none" className="shrink-0">
            <path d="M1 1L6 6L1 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>
        <div className="flex flex-col items-center gap-1.5 px-4 py-3 rounded-lg border border-dashed border-[var(--sl-border)] bg-canvas min-w-[80px]">
          <div className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold bg-[var(--sl-warning-light)] text-[var(--sl-warning)]">
            G
          </div>
          <span className="text-xs font-semibold text-[var(--sl-ink)]">Gate</span>
          <span className="text-[10px] font-medium text-[var(--sl-ink-secondary)]">
            {currentData.workItems.filter((wi) => wi.status === 'InReview').length} 待审
          </span>
        </div>
      </div>
    </div>
  );
};

export default WorkflowPanorama;
