import React, { useMemo, useState } from 'react';
import { UserPlus } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { getSeatRoleLabel } from '../utils/display';
import { SeatRole } from '../types';

interface AddSeatDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const ROLES: SeatRole[] = ['ProductOwner', 'Architect', 'Designer', 'Verifier'];

const AddSeatDialog: React.FC<AddSeatDialogProps> = ({ isOpen, onClose }) => {
  const { addSeat } = useDataStore();
  const [name, setName] = useState('');
  const [role, setRole] = useState<SeatRole>('Verifier');

  const handleCreate = () => {
    if (!name) return;
    addSeat({
      id: `seat-${Math.random().toString(36).substr(2, 4)}`,
      name,
      role,
      status: 'Active',
      created_at: new Date().toISOString(),
    });
    setName('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-secondary/40 backdrop-blur-sm animate-in fade-in duration-300">
      <div className="bg-card w-full max-w-lg rounded-3xl border border-border/60 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
        <div className="bg-primary/5 px-8 py-6 border-b border-border/40 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
              <UserPlus size={20} />
            </div>
            <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">新建席位 (NEW SEAT)</h2>
          </div>
        </div>

        <div className="p-8 space-y-6">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">席位名称</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如: nexus-architect"
              className="w-full bg-secondary/40 border border-border/60 rounded-xl px-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm"
            />
          </div>

          <div className="space-y-2">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">职责角色</label>
            <div className="grid grid-cols-2 gap-2">
              {ROLES.map((r) => {
                const roleKey = typeof r === 'string' ? r : `custom-${r.Custom}`;
                return (
                  <button
                    key={roleKey}
                    onClick={() => setRole(r)}
                    className={`px-4 py-2.5 rounded-xl border text-[10px] font-black tracking-widest transition-all ${
                      role === r ? 'bg-primary text-white border-primary shadow-lg shadow-primary/20' : 'bg-card border-border/60 text-text-secondary hover:bg-accent'
                    }`}
                  >
                    {getSeatRoleLabel(r)}
                  </button>
                );
              })}
            </div>
          </div>

          <p className="text-[11px] text-text-secondary leading-relaxed bg-accent/30 p-4 rounded-xl border border-primary/10 italic">
            建立席位后，您将能够为其分配特定的 WorkItem 或通过它启动新的运行时会话。席位身份在项目账本中具有持久性。
          </p>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-end gap-3">
          <button
            onClick={onClose}
            className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-text-secondary hover:text-text-primary transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleCreate}
            disabled={!name}
            className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-primary text-white rounded-xl hover:bg-primary-hover transition-all shadow-lg shadow-primary/20 disabled:opacity-50 disabled:grayscale disabled:shadow-none"
          >
            创建席位
          </button>
        </div>
      </div>
    </div>
  );
};

export default AddSeatDialog;
