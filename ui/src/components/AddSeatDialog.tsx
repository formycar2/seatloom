import React, { useMemo, useState } from 'react';
import { UserPlus } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { getSeatRoleLabel } from '../utils/display';

interface AddSeatDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const roleOptions = ['ProductOwner', 'Architect', 'Verifier', 'Designer', 'Developer'] as const;

const buildSeatId = (input: string) => {
  const slug = input
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  return `seat-${slug || Date.now()}`;
};

const AddSeatDialog: React.FC<AddSeatDialogProps> = ({ isOpen, onClose }) => {
  const { addSeat } = useDataStore();
  const [name, setName] = useState('');
  const [role, setRole] = useState<(typeof roleOptions)[number]>('Architect');

  const previewId = useMemo(() => buildSeatId(name), [name]);

  if (!isOpen) return null;

  const handleAdd = () => {
    if (!name.trim()) return;
    addSeat({
      id: previewId,
      name: name.trim(),
      role,
      status: 'Active',
      created_at: new Date().toISOString(),
    });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-card w-[440px] rounded-xl shadow-2xl border border-border overflow-hidden animate-in fade-in duration-200">
        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <h2 className="text-xl font-bold flex items-center gap-2 text-primary">
              <UserPlus size={20} />
              新增协作席位
            </h2>
            <p className="text-sm text-muted-foreground leading-relaxed">
              把新的执行角色登记到当前项目，后续它就可以被分配工作项、挂接会话、接收交接，避免临时协作只存在于终端上下文里。
            </p>
          </div>

          <div className="rounded-xl border border-border bg-secondary/40 p-4 space-y-2 text-xs text-muted-foreground leading-relaxed">
            <div className="font-semibold text-foreground">创建规则</div>
            <div>1. 席位名称用于界面展示，可填写中文或团队常用代号。</div>
            <div>2. 系统会自动生成可追踪 ID：<span className="font-mono text-foreground">{previewId}</span></div>
            <div>3. 角色决定该席位在摘要、交接与验收视图里的默认职责标签。</div>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">
              席位名称 <span className="text-status-error">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如：前端补位、交付复核、ops-review"
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 font-medium"
              autoFocus
            />
            <p className="text-[11px] text-muted-foreground">
              若你希望后续命令行、tmux 或交接单更稳定追踪，建议名称保持简短且可辨识。
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">职责角色</label>
            <select
              value={role}
              onChange={(e) => setRole(e.target.value as (typeof roleOptions)[number])}
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
            >
              {roleOptions.map((option) => (
                <option key={option} value={option}>
                  {getSeatRoleLabel(option)}
                </option>
              ))}
            </select>
            <p className="text-[11px] text-muted-foreground">
              当前选择：该席位会默认以“{getSeatRoleLabel(role)}”身份出现在团队视图与任务归属里。
            </p>
          </div>
        </div>

        <div className="p-4 bg-secondary/50 border-t border-border flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-muted-foreground hover:bg-black/5 rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleAdd}
            disabled={!name.trim()}
            className="px-6 py-2 text-xs font-bold uppercase tracking-widest bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-primary/20"
          >
            创建席位
          </button>
        </div>
      </div>
    </div>
  );
};

export default AddSeatDialog;
