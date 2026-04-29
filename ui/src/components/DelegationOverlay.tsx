import React, { useState } from 'react';
import {
  UserPlus,
  X,
  ShieldCheck,
  Calendar,
  AlertTriangle,
  Zap,
  Info,
  User,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { Seat, WorkItem } from '../types';
import { getSeatRoleLabel } from '../utils/display';

interface DelegationOverlayProps {
  workItem: WorkItem;
  sourceSeat: Seat;
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (delegation: any) => void;
}

const DelegationOverlay: React.FC<DelegationOverlayProps> = ({
  workItem,
  sourceSeat,
  isOpen,
  onClose,
  onConfirm,
}) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];
  
  const [delegateSeatId, setDelegateSeatId] = useState('');
  const [scope, setScope] = useState(`协助处理工作项 ${workItem.id.toUpperCase()} 的相关任务。`);
  const [issuerId, setIssuerId] = useState('Human'); 
  const [expiry, setExpiry] = useState(new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0]);
  const [authorityLimit, setAuthorityLimit] = useState('仅限非破坏性修改，涉及架构变动需汇报。');

  if (!isOpen) return null;

  const isValid = delegateSeatId && scope && issuerId && expiry && delegateSeatId !== sourceSeat.id;
  const isSelfDelegation = delegateSeatId === sourceSeat.id;

  const handleConfirm = () => {
    if (!isValid) return;
    
    const delegation = {
      source_seat_id: sourceSeat.id,
      delegate_seat_id: delegateSeatId,
      scope,
      issuer: issuerId === 'Human' ? 'Human' : { Seat: issuerId },
      expiry,
      authority_limit: authorityLimit,
      created_at: new Date().toISOString(),
    };
    
    onConfirm(delegation);
  };

  return (
    <div className="fixed inset-0 z-[150] flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-xl rounded-3xl border border-border shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/60 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <UserPlus size={24} />
            </div>
            <div>
              <h2 className="text-xl font-black tracking-tight text-ink uppercase">临时委派 (DELEGATION)</h2>
              <p className="text-[10px] font-bold text-ink-soft opacity-60 uppercase tracking-widest mt-0.5">WorkItem Scoped Responsibility</p>
            </div>
          </div>
          <button onClick={onClose} className="text-ink-faint hover:text-ink transition-colors p-1">
            <X size={20} />
          </button>
        </div>

        <div className="p-8 space-y-6">
          <div className="flex items-center gap-4 bg-accent/30 p-4 rounded-2xl border border-primary/10">
            <div className="flex-1 space-y-1">
              <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Target WorkItem</div>
              <div className="text-sm font-bold text-ink truncate">{workItem.title}</div>
            </div>
            <div className="h-8 w-px bg-primary/10" />
            <div className="flex-1 space-y-1">
              <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Source Owner</div>
              <div className="text-sm font-bold text-ink">{sourceSeat.name}</div>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">委派目标席位</label>
              <select
                value={delegateSeatId}
                onChange={(e) => setDelegateSeatId(e.target.value)}
                className={`w-full bg-secondary/40 border rounded-xl px-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm ${
                  isSelfDelegation ? 'border-status-error' : 'border-border/60'
                }`}
              >
                <option value="">选择目标席位...</option>
                {seats.map((seat) => (
                  <option key={seat.id} value={seat.id}>{seat.name} ({getSeatRoleLabel(seat.role)})</option>
                ))}
              </select>
              {isSelfDelegation && (
                <p className="text-[10px] text-status-error font-bold flex items-center gap-1 mt-1 ml-1">
                  <AlertTriangle size={10} /> 不能委派给原负责人。
                </p>
              )}
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">签发人 (ISSUER)</label>
              <select
                value={issuerId}
                onChange={(e) => setIssuerId(e.target.value)}
                className="w-full bg-secondary/40 border border-border/60 rounded-xl px-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm"
              >
                <option value="Human">人工 (Supervisor)</option>
                {seats.map((seat) => (
                  <option key={seat.id} value={seat.id}>{seat.name} ({getSeatRoleLabel(seat.role)})</option>
                ))}
              </select>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">有效期至 (EXPIRY)</label>
              <div className="relative">
                <Calendar className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-ink-faint" />
                <input
                  type="date"
                  value={expiry}
                  onChange={(e) => setExpiry(e.target.value)}
                  className="w-full bg-secondary/40 border border-border/60 rounded-xl pl-10 pr-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm"
                />
              </div>
            </div>

            <div className="space-y-2">
              <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">委派范围说明</label>
              <textarea
                value={scope}
                onChange={(e) => setScope(e.target.value)}
                className="w-full h-12 bg-secondary/40 border border-border/60 rounded-xl px-4 py-2 text-sm font-medium focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm resize-none"
                placeholder="明确定义临时委派的工作范围..."
              />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-ink-soft uppercase tracking-widest ml-1">权限限制 (AUTHORITY LIMIT)</label>
            <div className="relative">
              <ShieldCheck className="absolute left-3 top-3 w-4 h-4 text-primary opacity-60" />
              <textarea
                value={authorityLimit}
                onChange={(e) => setAuthorityLimit(e.target.value)}
                className="w-full h-20 bg-primary/5 border border-primary/20 rounded-2xl pl-10 pr-4 py-3 text-sm font-medium focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm resize-none"
                placeholder="设置具体的权限边界..."
              />
            </div>
          </div>

          <div className="flex items-start gap-3 p-3 bg-secondary/30 rounded-xl border border-border/40">
            <Info size={16} className="text-ink-soft shrink-0 mt-0.5" />
            <p className="text-[11px] text-ink-soft leading-relaxed italic opacity-80">
              提示：临时委派不改变工作项的原始所有权。委派结束后，决策权将自动收回到原负责人席位。
            </p>
          </div>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-between items-center">
          <button onClick={onClose} className="text-[10px] font-black text-text-secondary hover:text-text-primary transition-colors uppercase tracking-widest">
            取消 (CANCEL)
          </button>
          <button
            onClick={handleConfirm}
            disabled={!isValid}
            title={!isValid ? (isSelfDelegation ? '不能委派给原负责人' : '请填写完整的委派范围、有效期及签发人信息') : '确认执行临时委派'}
            className="px-8 py-3 bg-primary text-surface rounded-xl text-[10px] font-black uppercase tracking-widest flex items-center gap-2 shadow-xl shadow-primary/20 hover:brightness-110 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:shadow-none"
          >
            <Zap size={14} />
            确认临时委派 (CONFIRM DELEGATION)
          </button>
        </div>
      </div>
    </div>
  );
};

export default DelegationOverlay;
