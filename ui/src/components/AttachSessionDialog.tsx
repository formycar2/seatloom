import React, { useMemo, useState } from 'react';
import { Link, Monitor } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { Session, Runtime } from '../types';
import { getRuntimeLabel, getSeatRoleLabel } from '../utils/display';

interface AttachSessionDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const detectedProcesses: Array<{
  pid: string;
  runtime: Runtime;
  branch: string;
  workspace: string;
  command: string;
  note: string;
}> = [
  {
    pid: '42951',
    runtime: 'Codex',
    branch: 'demo/zh-density-pass',
    workspace: '~/Documents/GitHub/seatloom/ui',
    command: 'codex --resume ses-406',
    note: 'Lyra 正在补中文高密度数据与文案对账。',
  },
  {
    pid: '42988',
    runtime: 'CursorCli',
    branch: 'repair/sg-01-verify',
    workspace: '~/Documents/GitHub/seatloom',
    command: 'cursor --wait repair/sg-01-verify',
    note: 'Flux 保留的验证终端，可用于复核构建与验收证据。',
  },
];

const AttachSessionDialog: React.FC<AttachSessionDialogProps> = ({ isOpen, onClose }) => {
  const { addSession, activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];
  const [selectedPid, setSelectedPid] = useState(detectedProcesses[0].pid);
  const [seatId, setSeatId] = useState(seats[0]?.id || '');

  const selectedProcess = useMemo(
    () => detectedProcesses.find((process) => process.pid === selectedPid) || detectedProcesses[0],
    [selectedPid]
  );

  if (!isOpen) return null;

  const handleAttach = () => {
    if (!seatId || !selectedProcess) return;

    const newSession: Session = {
      id: `ses-${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
      seat_id: seatId,
      runtime: selectedProcess.runtime,
      workspace_path: selectedProcess.workspace,
      branch: selectedProcess.branch,
      status: 'Running',
      pid: parseInt(selectedPid, 10),
      created_at: new Date().toISOString(),
    };
    addSession(newSession);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-card w-[520px] rounded-xl shadow-2xl border border-border overflow-hidden animate-in fade-in duration-200">
        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <h2 className="text-xl font-bold flex items-center gap-2">
              <Link size={20} className="text-primary" />
              接入已有运行进程
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              将本地已经在运行的代理进程重新纳入 SeatLoom 追踪，避免重开会话后丢失上下文、分支和执行状态。
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">检测到的进程</label>
            <div className="space-y-2 max-h-56 overflow-y-auto border border-border rounded-lg p-2 bg-secondary/30">
              {detectedProcesses.map((process) => (
                <label
                  key={process.pid}
                  className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                    selectedPid === process.pid ? 'border-primary bg-primary/10' : 'border-border hover:bg-black/5'
                  }`}
                >
                  <input
                    type="radio"
                    name="pid"
                    value={process.pid}
                    checked={selectedPid === process.pid}
                    onChange={() => setSelectedPid(process.pid)}
                    className="mt-1 text-primary"
                  />
                  <Monitor size={14} className="text-muted-foreground mt-0.5" />
                  <div className="flex-1 space-y-1 min-w-0">
                    <div className="flex items-center justify-between gap-3 flex-wrap">
                      <div className="text-xs font-semibold text-foreground">PID {process.pid} · {getRuntimeLabel(process.runtime)}</div>
                      <div className="text-[11px] text-muted-foreground">分支：{process.branch}</div>
                    </div>
                    <div className="text-xs font-mono text-foreground break-all">{process.command}</div>
                    <div className="text-[11px] text-muted-foreground leading-relaxed">{process.note}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">挂接到席位</label>
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
            <p className="text-[11px] text-muted-foreground">
              当前将以 {selectedProcess ? getRuntimeLabel(selectedProcess.runtime) : '未知运行时'} 继续写入时间线、终端实况和工作项关联信息。
            </p>
          </div>
        </div>

        <div className="p-4 border-t border-border bg-secondary flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm font-medium hover:bg-black/5 rounded-lg transition-colors">取消</button>
          <button
            onClick={handleAttach}
            disabled={!seatId}
            className="px-6 py-2 text-sm font-bold bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-primary/20 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            接入并追踪
          </button>
        </div>
      </div>
    </div>
  );
};

export default AttachSessionDialog;
