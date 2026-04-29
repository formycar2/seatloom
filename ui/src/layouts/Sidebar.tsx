import React from 'react';
import { Circle, Plus, User } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import { getPriorityLabel, getRuntimeLabel, getSeatRoleLabel, getSessionStatusLabel, getWorkItemStatusLabel } from '../utils/display';
import { useAppStore } from '../stores/useAppStore';

interface SidebarProps {
  activeObjectId: string | null;
  onSelectObject: (type: any, object: any) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ activeObjectId, onSelectObject }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const {} = useAppStore();
  
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentData) return null;

  const { seats, sessions, workItems } = currentData;

  return (
    <div className="w-[260px] bg-secondary border-r border-border flex flex-col py-3 overflow-hidden animate-in slide-in-from-left duration-300">
      <div className="flex-1 overflow-y-auto px-2 space-y-6">
        <div>
          <div className="px-3 mb-2 flex items-center justify-between">
            <span className="text-xs font-bold text-text-muted tracking-wide uppercase">{t.sidebar.seats}</span>
            <button className="hover:bg-primary/10 hover:rounded-full p-1 text-text-muted hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary" tabIndex={0} onClick={() => onSelectObject('Action', 'AddSeat')}>
              <Plus size={12} />
            </button>
          </div>

          <div className="space-y-2">
            {seats.map((seat) => {
              const seatSessions = sessions.filter((session) => session.seat_id === seat.id);
              const isActive = activeObjectId === seat.id;

              return (
                <div key={seat.id} className="space-y-1">
                  <div onClick={() => onSelectObject('Seat', seat)} className={`sl-clickable px-3 py-2 flex items-center gap-3 ${isActive ? 'bg-primary text-surface font-bold shadow-md' : ''}`} tabIndex={0}>
                    <div className={`p-1.5 rounded-md shadow-sm border ${isActive ? 'bg-surface/10 border-surface/20' : 'bg-background border-border'}`}>
                      <User size={12} className={isActive ? 'text-surface' : 'text-text-secondary'} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className={`text-sm font-semibold truncate leading-tight mb-0.5 flex items-center gap-1.5 ${isActive ? 'text-surface' : 'text-text-primary'}`}>
                        {seat.name}
                        <Circle size={6} className={`fill-current ${seat.status === 'Active' ? 'text-status-active' : 'text-status-done'}`} />
                      </div>
                      <div className={`text-[11px] font-medium tracking-tight ${isActive ? 'text-surface/80' : 'text-text-secondary'}`}>{getSeatRoleLabel(seat.role)}</div>
                    </div>
                  </div>

                  {seatSessions.length > 0 && (
                    <div className="pl-9 space-y-1 border-l border-border/50 ml-5 mt-1">
                      {seatSessions.map((session) => {
                        const isSessionActive = activeObjectId === session.id;
                        const isBlocked = session.status === 'InputRequired' || !!session.prompt_state;
                        const statusColor =
                          session.status === 'Running'
                            ? 'text-status-active'
                            : isBlocked
                              ? 'text-status-warning'
                              : session.status === 'Interrupted'
                                ? 'text-status-error'
                                : 'text-status-done';

                        return (
                          <div
                            key={session.id}
                            onClick={() => onSelectObject('Session', session)}
                            className={`sl-clickable px-2 py-1.5 flex items-center justify-between gap-3 ${isSessionActive ? 'bg-primary text-surface font-bold shadow-md' : 'text-text-secondary'}`}
                            tabIndex={0}
                          >
                            <div className="flex items-center gap-2 min-w-0 flex-1">
                              <Circle size={6} className={`fill-current ${statusColor} shrink-0`} />
                              <div className="min-w-0 flex-1">
                                <div className="flex items-center gap-1.5">
                                  <div className={`monospace text-[11px] truncate ${isSessionActive ? 'text-surface' : 'text-text-primary'}`}>{session.id}</div>
                                  {isBlocked && (
                                    <span className={`px-1 py-0.5 rounded text-[8px] font-black uppercase tracking-tighter shrink-0 border ${isSessionActive ? 'bg-surface text-primary border-surface/20' : 'bg-status-warning text-surface border-warning/20'}`}>
                                      Prompt
                                    </span>
                                  )}
                                </div>
                                <div className={`text-[10px] truncate ${isSessionActive ? 'text-surface/80' : 'text-text-secondary'}`}>{getSessionStatusLabel(session.status)}</div>
                              </div>
                            </div>
                            <span className={`text-[10px] text-right max-w-[64px] truncate ${isSessionActive ? 'text-surface/60' : 'text-text-muted'}`}>{getRuntimeLabel(session.runtime)}</span>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        <div>
          <div className="px-3 mb-2 flex items-center justify-between">
            <span className="text-xs font-bold text-text-muted tracking-wide uppercase">{t.sidebar.workitems}</span>
            <button className="hover:bg-primary/10 hover:rounded-full p-1 text-text-muted hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded" tabIndex={0} onClick={() => onSelectObject('Action', 'AddWorkItem')}>
              <Plus size={12} />
            </button>
          </div>

          <div className="space-y-1">
            {workItems.map((workItem) => {
              const isActive = activeObjectId === workItem.id;
              const statusColor =
                workItem.status === 'Active'
                  ? 'text-status-active'
                  : workItem.status === 'Blocked'
                    ? 'text-status-error'
                    : workItem.status === 'Done'
                      ? 'text-status-done'
                      : workItem.priority === 'High' || workItem.priority === 'Critical'
                        ? 'text-status-error'
                        : 'text-status-warning';

              return (
                <div key={workItem.id} onClick={() => onSelectObject('WorkItem', workItem)} className={`sl-clickable px-3 py-2 flex flex-col gap-1 ${isActive ? 'bg-primary text-surface font-bold shadow-md' : ''}`} tabIndex={0}>
                  <div className="flex items-center justify-between min-w-0">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <Circle size={6} className={`fill-current ${statusColor}`} />
                      <span className={`monospace text-[11px] font-black ${isActive ? 'text-surface' : 'text-text-primary'}`}>{workItem.id.toUpperCase()}</span>
                    </div>
                    <span className={`text-[10px] font-bold ${isActive ? 'text-surface/70' : workItem.priority === 'High' || workItem.priority === 'Critical' ? 'text-status-error' : 'text-text-secondary'}`}>
                      {getPriorityLabel(workItem.priority)}
                    </span>
                  </div>
                  <div className={`text-xs font-medium leading-snug line-clamp-2 ${isActive ? 'text-surface' : 'text-text-primary'}`}>{workItem.title}</div>
                  <div className={`pl-3 border-l-2 ${isActive ? 'border-surface/40' : 'border-primary/20'} text-[10px] flex items-center justify-between gap-2`}>
                    <span className={`truncate ${isActive ? 'text-surface/70' : 'text-text-secondary'}`}>{getWorkItemStatusLabel(workItem.status)}</span>
                    {workItem.owner_seat_id && <span className={`truncate ${isActive ? 'text-surface/70' : 'text-text-secondary'}`}>负责人：{seats.find((seat) => seat.id === workItem.owner_seat_id)?.name || workItem.owner_seat_id}</span>}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div className="px-3 pt-3 mt-auto">
        <div className="flex flex-col gap-2">
          {/* Theme selector moved to system configuration button */}
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
