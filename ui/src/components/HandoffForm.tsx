import React, { useState } from 'react';
import { Plus, X } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';

interface HandoffFormProps {
  onClose: () => void;
  workItemId?: string;
}

const HandoffForm: React.FC<HandoffFormProps> = ({ onClose, workItemId: initialWorkItemId }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];
  const workItems = (currentData?.workItems || []).filter(wi => wi.status !== 'Done');

  const [toSeatId, setToSeatId] = useState('');
  const [workItemId, setWorkItemId] = useState(initialWorkItemId || '');
  const [purpose, setPurpose] = useState('');
  const [expectedOutcome, setExpectedOutcome] = useState('');

  if (!currentData) return null;

  return (
    <div className="flex flex-col h-full bg-background animate-in slide-in-from-right duration-300">
      <div className="p-6 border-b border-border/40 bg-card flex items-center justify-between shadow-sm">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-primary/10 text-primary rounded-xl border border-primary/20">
            <Plus size={20} />
          </div>
          <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">发起交接 (NEW HANDOFF)</h2>
        </div>
        <button onClick={onClose} className="p-1 hover:bg-accent text-text-secondary rounded-lg transition-colors">
          <X size={20} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-8 space-y-8">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="space-y-2">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">接收席位</label>
            <select
              value={toSeatId}
              onChange={(e) => setToSeatId(e.target.value)}
              className="w-full bg-secondary/40 border border-border/60 rounded-xl px-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm"
            >
              <option value="">请选择接收方...</option>
              {seats.map((seat) => (
                <option key={seat.id} value={seat.id}>{seat.name}</option>
              ))}
            </select>
          </div>
          <div className="space-y-2">
            <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">关联工作项</label>
            <select
              value={workItemId}
              onChange={(e) => setWorkItemId(e.target.value)}
              className="w-full bg-secondary/40 border border-border/60 rounded-xl px-4 py-3 text-sm font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm"
            >
              <option value="">请选择工作项...</option>
              {workItems.map((wi) => (
                <option key={wi.id} value={wi.id}>{wi.id.toUpperCase()} - {wi.title}</option>
              ))}
            </select>
          </div>
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">交接目的</label>
          <textarea
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            placeholder="说明为什么要进行这次交接，以及接收方需要关注的重点。"
            className="w-full h-32 bg-secondary/40 border border-border/60 rounded-2xl px-4 py-3 text-sm font-medium focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm resize-none"
          />
        </div>

        <div className="space-y-2">
          <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">预期交付产物</label>
          <textarea
            value={expectedOutcome}
            onChange={(e) => setExpectedOutcome(e.target.value)}
            placeholder="明确定义交接完成后的交付标准，例如：'一套通过验证的 UI 组件'。"
            className="w-full h-24 bg-secondary/40 border border-border/60 rounded-2xl px-4 py-3 text-sm font-medium focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all shadow-sm resize-none"
          />
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">附加证据</label>
          <div className="border-2 border-dashed border-border/60 rounded-2xl p-6 flex flex-col items-center justify-center gap-2 hover:border-primary/40 hover:bg-accent cursor-pointer transition-all group shadow-inner">
            <div className="p-3 bg-secondary rounded-full text-text-secondary group-hover:bg-card group-hover:text-primary transition-colors">
              <Plus size={24} />
            </div>
            <div className="text-[10px] font-black text-text-secondary uppercase tracking-widest group-hover:text-primary transition-colors">附加文档证据 (ATTACH EVIDENCE)</div>
          </div>
        </div>

        <div className="bg-secondary/30 px-8 py-6 border-t border-border/40 flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-[10px] font-black uppercase tracking-widest text-text-secondary hover:text-text-primary transition-colors">
            取消 (CANCEL)
          </button>
          <button
            onClick={onClose}
            className="px-6 py-2 text-[10px] font-black uppercase tracking-widest bg-status-active text-white rounded-xl hover:opacity-90 transition-all shadow-lg shadow-status-active/20 disabled:opacity-50 disabled:grayscale flex items-center gap-2"
          >
            发送交接单 (SEND HANDOFF)
          </button>
        </div>
      </div>
    </div>
  );
};

export default HandoffForm;
