import React, { useMemo } from 'react';
import { Search, User, Circle, Cpu, ChevronRight } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { getSeatRoleLabel, getSessionStatusLabel } from '../utils/display';
import { Seat, Session } from '../types';

interface SeatsViewProps {
  onSelectSeat: (seat: Seat) => void;
  onSelectSession: (session: Session) => void;
  selectedId?: string | null;
}

const statusColors: Record<string, string> = {
  Active: 'bg-[var(--sl-success)]',
  Paused: 'bg-[var(--sl-warning)]',
  Archived: 'bg-[var(--sl-done)]',
};

const sessionStatusColors: Record<string, string> = {
  Running: 'text-[var(--sl-success)]',
  InputRequired: 'text-[var(--sl-error)]',
  Completed: 'text-[var(--sl-done)]',
  Failed: 'text-[var(--sl-error)]',
  Suspended: 'text-[var(--sl-warning)]',
  Interrupted: 'text-[var(--sl-warning)]',
  Launching: 'text-primary',
};

const SeatsView: React.FC<SeatsViewProps> = ({ onSelectSeat, onSelectSession, selectedId }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentData) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)]">
        <p className="text-sm">请选择一个项目</p>
      </div>
    );
  }

  const { seats, sessions } = currentData;

  return (
    <div className="flex-1 overflow-y-auto">
      {/* Seats section */}
      <div className="px-3 py-2 border-b border-[var(--sl-border-subtle)]">
        <span className="text-[11px] font-semibold text-[var(--sl-ink-muted)] uppercase tracking-wider">
          席位 ({seats.length})
        </span>
      </div>

      <div className="divide-y divide-[var(--sl-border-subtle)]">
        {seats.map((seat) => {
          const seatSessions = sessions.filter((s) => s.seat_id === seat.id);
          const isSelected = selectedId === seat.id;

          return (
            <div key={seat.id}>
              <button
                onClick={() => onSelectSeat(seat)}
                className={`w-full flex items-center gap-3 px-4 py-3 text-left transition-colors
                  ${isSelected ? 'bg-primary/5 border-l-[3px] border-l-primary' : 'hover:bg-[var(--sl-panel)] border-l-[3px] border-l-transparent'}
                `}
              >
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                  <User size={16} className="text-primary" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-semibold text-[var(--sl-ink)] truncate">
                      {seat.name.charAt(0).toUpperCase() + seat.name.slice(1)}
                    </span>
                    <div className={`w-2 h-2 rounded-full ${statusColors[seat.status] || 'bg-[var(--sl-done)]'}`} />
                  </div>
                  <span className="text-xs text-[var(--sl-ink-secondary)]">
                    {getSeatRoleLabel(seat.role)}
                  </span>
                </div>
                <ChevronRight size={14} className="text-[var(--sl-ink-muted)] shrink-0" />
              </button>

              {/* Nested sessions */}
              {seatSessions.length > 0 && (
                <div className="pl-12 bg-[var(--sl-panel)]/50">
                  {seatSessions.map((session) => (
                    <button
                      key={session.id}
                      onClick={() => onSelectSession(session)}
                      className={`w-full flex items-center gap-2 px-3 py-2 text-left transition-colors
                        ${selectedId === session.id ? 'bg-primary/5' : 'hover:bg-[var(--sl-panel)]'}
                      `}
                    >
                      <Cpu size={12} className={sessionStatusColors[session.status] || 'text-[var(--sl-ink-muted)]'} />
                      <span className="text-xs text-[var(--sl-ink)] truncate flex-1">{session.id}</span>
                      <span className={`text-[10px] font-medium ${sessionStatusColors[session.status] || 'text-[var(--sl-ink-muted)]'}`}>
                        {getSessionStatusLabel(session.status)}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default SeatsView;
