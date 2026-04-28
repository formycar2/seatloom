import React from 'react';
import { Circle, Plus, User } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import { getPriorityLabel, getRuntimeLabel, getSeatRoleLabel, getSessionStatusLabel, getWorkItemStatusLabel } from '../utils/display';

interface SidebarProps {
  activeObjectId: string | null;
  onSelectObject: (type: string, data: any) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ activeObjectId, onSelectObject }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  if (!currentData) return null;

  const { seats, sessions, workItems } = currentData;

  return (
    <div className="w-[260px] bg-secondary/80 border-r border-border flex flex-col py-3 overflow-hidden animate-in slide-in-from-left duration-300">
      <div className="flex-1 overflow-y-auto px-2 space-y-6">
        <div>
          <div className="px-3 mb-2 flex items-center justify-between">
            <span className="text-xs font-bold text-muted-foreground tracking-wide">{t.sidebar.seats}</span>
            <button className="hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded" tabIndex={0} onClick={() => onSelectObject('Action', 'AddSeat')}>
              <Plus size={12} />
            </button>
          </div>

          <div className="space-y-2">
            {seats.map((seat) => {
              const seatSessions = sessions.filter((session) => session.seat_id === seat.id);
              const isActive = activeObjectId === seat.id;

              return (
                <div key={seat.id} className="space-y-1">
                  <div onClick={() => onSelectObject('Seat', seat)} className={`sl-clickable px-3 py-2 flex items-center gap-3 ${isActive ? 'bg-primary/5 text-primary' : ''}`} tabIndex={0}>
                    <div className="p-1.5 bg-background border border-border rounded-md shadow-sm">
                      <User size={12} className={isActive ? 'text-primary' : 'text-muted-foreground'} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-semibold truncate leading-tight mb-1 flex items-center gap-1.5">
                        {seat.name}
                        <Circle size={6} className={`fill-current ${seat.status === 'Active' ? 'text-status-active' : 'text-status-done'}`} />
                      </div>
                      <div className="text-[11px] text-muted-foreground font-medium tracking-tight">{getSeatRoleLabel(seat.role)}</div>
                    </div>
                  </div>

                  {seatSessions.length > 0 && (
                    <div className="pl-9 space-y-1 border-l border-border/50 ml-5 mt-1">
                      {seatSessions.map((session) => {
                        const isSessionActive = activeObjectId === session.id;
                        const statusColor =
                          session.status === 'Running'
                            ? 'text-status-active'
                            : session.status === 'InputRequired'
                              ? 'text-status-warning'
                              : session.status === 'Interrupted'
                                ? 'text-status-error'
                                : 'text-status-done';

                        return (
                          <div
                            key={session.id}
                            onClick={() => onSelectObject('Session', session)}
                            className={`sl-clickable px-2 py-1.5 flex items-center justify-between gap-3 ${isSessionActive ? 'text-primary font-bold' : 'text-muted-foreground'}`}
                            tabIndex={0}
                          >
                            <div className="flex items-center gap-2 min-w-0">
                              <Circle size={6} className={`fill-current ${statusColor}`} />
                              <div className="min-w-0">
                                <div className="monospace text-[11px] truncate">{session.id}</div>
                                <div className="text-[10px] truncate opacity-70">{getSessionStatusLabel(session.status)}</div>
                              </div>
                            </div>
                            <span className="text-[10px] opacity-60 text-right max-w-[72px] truncate">{getRuntimeLabel(session.runtime)}</span>
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
            <span className="text-xs font-bold text-muted-foreground tracking-wide">{t.sidebar.workitems}</span>
            <button className="hover:text-primary transition-colors focus:outline-none focus:ring-1 focus:ring-primary rounded" tabIndex={0} onClick={() => onSelectObject('Action', 'AddWorkItem')}>
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
                <div key={workItem.id} onClick={() => onSelectObject('WorkItem', workItem)} className={`sl-clickable px-3 py-2 flex flex-col gap-1 ${isActive ? 'bg-primary/5' : ''}`} tabIndex={0}>
                  <div className="flex items-center justify-between min-w-0">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <Circle size={6} className={`fill-current ${statusColor}`} />
                      <span className={`monospace text-[11px] font-semibold ${isActive ? 'text-primary' : 'text-foreground'}`}>{workItem.id.toUpperCase()}</span>
                    </div>
                    <span className={`text-[10px] font-semibold ${workItem.priority === 'High' || workItem.priority === 'Critical' ? 'text-status-error' : 'text-muted-foreground'}`}>
                      {getPriorityLabel(workItem.priority)}
                    </span>
                  </div>
                  <div className="text-xs font-medium text-foreground leading-snug line-clamp-2">{workItem.title}</div>
                  <div className="pl-3 border-l-2 border-transparent text-[10px] text-muted-foreground flex items-center justify-between gap-2">
                    <span className="truncate">{getWorkItemStatusLabel(workItem.status)}</span>
                    {workItem.owner_seat_id && <span className="truncate">负责人：{seats.find((seat) => seat.id === workItem.owner_seat_id)?.name || workItem.owner_seat_id}</span>}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Sidebar;
