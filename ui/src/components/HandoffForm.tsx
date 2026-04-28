import React, { useState } from 'react';
import { Forward, Paperclip, CheckCircle } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import { getSeatRoleLabel, getWorkItemStatusLabel } from '../utils/display';

interface HandoffFormProps {
  onClose: () => void;
  workItemId?: string;
}

const HandoffForm: React.FC<HandoffFormProps> = ({ onClose, workItemId }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData, addHandoff } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];
  const workItems = currentData?.workItems || [];

  const currentWi = workItems.find((wi) => wi.id === workItemId) || workItems[0] || null;
  const senderSeatId = currentWi?.owner_seat_id || seats[0]?.id || 'seat-unassigned';

  const [recipient, setRecipient] = useState(seats[1]?.id || 'Human');
  const [purpose, setPurpose] = useState('');
  const [expected, setExpected] = useState('');
  const [receipt, setReceipt] = useState(true);

  const handleSubmit = () => {
    if (!purpose.trim() || !expected.trim() || !currentWi) return;

    addHandoff({
      id: `ho-${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
      from_ref: { Seat: senderSeatId },
      to_ref: recipient === 'Human' ? 'Human' : { Seat: recipient },
      workitem_id: currentWi.id,
      purpose: purpose.trim(),
      expected_outcome: expected.trim(),
      artifact_ids: [],
      required_receipt: receipt,
      status: 'Sent',
      created_at: new Date().toISOString(),
      sent_at: new Date().toISOString(),
    });

    onClose();
  };

  return (
    <div className="flex flex-col h-full bg-background animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-center justify-between p-4 border-b border-border bg-card">
        <div className="space-y-1">
          <h2 className="text-lg font-bold flex items-center gap-2 text-status-active">
            <Forward size={20} />
            {t.forms.createHandoff}
          </h2>
          <p className="text-xs text-muted-foreground">
            只有把目标、边界、回传条件写清楚，交接才不会退化成“把问题扔给下一个席位”。
          </p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {currentWi ? (
          <div className="rounded-xl border border-border bg-secondary/40 p-4 space-y-2">
            <div className="flex items-center justify-between gap-3 flex-wrap">
              <div>
                <div className="text-[11px] text-muted-foreground">当前交接关联工作项</div>
                <div className="text-sm font-semibold text-foreground">{currentWi.id} · {currentWi.title}</div>
              </div>
              <span className="text-[11px] px-2 py-1 rounded-full bg-background border border-border text-foreground">
                {getWorkItemStatusLabel(currentWi.status)}
              </span>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed">
              {currentWi.goal || '当前工作项尚未补充目标说明，建议在交接前先完善目标和验收口径。'}
            </p>
          </div>
        ) : (
          <div className="rounded-xl border border-status-warning/30 bg-status-warning/10 p-4 text-sm text-status-warning">
            当前没有可关联的工作项。请先创建工作项，再发起交接。
          </div>
        )}

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.recipient}</label>
          <select
            value={recipient}
            onChange={(e) => setRecipient(e.target.value)}
            className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary font-medium"
          >
            <option value="Human">人工介入（直接交给人处理）</option>
            {seats.map((seat) => (
              <option key={seat.id} value={seat.id}>
                {seat.name}（{typeof seat.role === 'string' ? getSeatRoleLabel(seat.role) : seat.role.Custom}）
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">
            {t.forms.purpose} <span className="text-status-error">*</span>
          </label>
          <textarea
            rows={3}
            value={purpose}
            onChange={(e) => setPurpose(e.target.value)}
            placeholder="请说明为什么现在要交接：例如“需要 Flux 代 Mira 完成字体/字号统一，且不能改动 Sidebar/Main 信息架构”。"
            className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary resize-none font-medium"
            autoFocus
          />
        </div>

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">
            {t.forms.expected} <span className="text-status-error">*</span>
          </label>
          <textarea
            rows={3}
            value={expected}
            onChange={(e) => setExpected(e.target.value)}
            placeholder="请写清楚什么算交接完成：例如“回传前后差异说明、构建通过、并在 `docs/coordination/` 写入产物路径”。"
            className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary resize-none"
          />
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between gap-3">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.attachArtifacts}</label>
            <span className="text-[11px] text-muted-foreground">当前演示态以文档路径与证据编号为主</span>
          </div>
          <div className="border-2 border-dashed border-border rounded-lg p-6 flex flex-col items-center justify-center gap-2 hover:border-primary/40 hover:bg-black/5 cursor-pointer transition-all group">
            <Paperclip size={24} className="text-muted-foreground group-hover:text-primary transition-colors" />
            <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
              从台账拖入产物，或在交接说明中写入证据路径
            </span>
            <span className="text-[11px] text-muted-foreground text-center leading-relaxed">
              例如：`docs/coordination/reviews/...`、`.local/evidence/...`、`ui/src/...`
            </span>
          </div>
        </div>

        <div className="flex items-center gap-3 p-3 bg-secondary/50 rounded-lg border border-border">
          <input type="checkbox" id="receipt" checked={receipt} onChange={(e) => setReceipt(e.target.checked)} className="rounded border-border text-primary focus:ring-primary" />
          <label htmlFor="receipt" className="text-[11px] cursor-pointer select-none font-bold uppercase tracking-wider flex items-center gap-2 text-foreground">
            <CheckCircle size={14} className={receipt ? 'text-status-active' : 'text-muted-foreground'} />
            {t.forms.receiptRequired}
          </label>
        </div>
      </div>

      <div className="p-4 border-t border-border bg-card flex justify-end gap-3">
        <button onClick={onClose} className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-muted-foreground hover:bg-black/5 rounded-lg transition-colors">
          {t.common.cancel}
        </button>
        <button
          onClick={handleSubmit}
          disabled={!purpose.trim() || !expected.trim() || !currentWi}
          className="px-6 py-2 text-xs font-bold uppercase tracking-widest bg-status-active text-white rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-status-active/20 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          发送交接 <Forward size={14} />
        </button>
      </div>
    </div>
  );
};

export default HandoffForm;
