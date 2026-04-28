import React, { useState } from 'react';
import { Terminal as TerminalIcon, CheckCircle } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { Session, Runtime } from '../types';
import { getRuntimeLabel, getSeatRoleLabel } from '../utils/display';

interface WrapLaunchDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const runtimeOptions: Runtime[] = ['ClaudeCode', 'Codex', 'CursorCli', 'GeminiCli'];

const WrapLaunchDialog: React.FC<WrapLaunchDialogProps> = ({ isOpen, onClose }) => {
  const { addSession, activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];

  const [seatId, setSeatId] = useState(seats[0]?.id || '');
  const [runtime, setRuntime] = useState<Runtime>('ClaudeCode');
  const [workspace, setWorkspace] = useState('~/Documents/GitHub/seatloom');

  if (!isOpen) return null;

  const handleLaunch = () => {
    if (!seatId) return;

    const newSession: Session = {
      id: `ses-${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
      seat_id: seatId,
      runtime,
      workspace_path: workspace,
      status: 'Launching',
      created_at: new Date().toISOString(),
    };
    addSession(newSession);
    setTimeout(() => {
      useDataStore.getState().updateSession(newSession.id, { status: 'Running', pid: Math.floor(Math.random() * 50000) });
    }, 1000);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-card w-[520px] rounded-xl shadow-2xl border border-border overflow-hidden animate-in fade-in duration-200">
        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <h2 className="text-xl font-bold flex items-center gap-2">
              <TerminalIcon size={20} className="text-primary" />
              启动新会话
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              以包裹式方式启动新的代理运行时，并把席位、工作目录、运行态和后续写回链路一次性登记到 SeatLoom。
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">目标席位</label>
            <select
              value={seatId}
              onChange={(e) => setSeatId(e.target.value)}
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
            >
              {seats.map((seat) => (
                <option key={seat.id} value={seat.id}>
                  {seat.name}（{typeof seat.role === 'string' ? getSeatRoleLabel(seat.role) : seat.role.Custom}）
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">运行时</label>
            <select
              value={runtime as string}
              onChange={(e) => setRuntime(e.target.value as Runtime)}
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
            >
              {runtimeOptions.map((option) => (
                <option key={option as string} value={option as string}>
                  {getRuntimeLabel(option)}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">工作目录</label>
            <input
              value={workspace}
              onChange={(e) => setWorkspace(e.target.value)}
              type="text"
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>

          <div className="bg-secondary/50 border border-border rounded-lg p-3 flex items-start gap-2">
            <CheckCircle size={14} className="text-status-active mt-0.5" />
            <p className="text-xs text-muted-foreground leading-relaxed">
              SeatLoom 会自动记录会话 PID、工作目录、运行时与启动时间，并尽量匹配原生会话标识，方便后续在收件箱、活动时间线、恢复包与验收证据之间形成闭环。
            </p>
          </div>
        </div>

        <div className="p-4 border-t border-border bg-secondary flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm font-medium hover:bg-black/5 rounded-lg transition-colors">取消</button>
          <button
            onClick={handleLaunch}
            disabled={!seatId || !workspace.trim()}
            className="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-primary/20 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            启动并登记
          </button>
        </div>
      </div>
    </div>
  );
};

export default WrapLaunchDialog;
