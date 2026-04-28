import React, { useMemo, useState } from 'react';
import { Plus, X, ListTodo } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';
import { useDataStore } from '../stores/useDataStore';
import { WorkItem } from '../types';
import { getPriorityLabel, getWorkItemStatusLabel } from '../utils/display';

interface WorkItemFormProps {
  onClose: () => void;
}

const priorityOptions: WorkItem['priority'][] = ['Low', 'Medium', 'High', 'Critical'];

const WorkItemForm: React.FC<WorkItemFormProps> = ({ onClose }) => {
  const { t } = useLocaleStore();
  const { addWorkItem, activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const seats = currentData?.seats || [];

  const [title, setTitle] = useState('');
  const [goal, setGoal] = useState('');
  const [priority, setPriority] = useState<WorkItem['priority']>('Medium');
  const [owner, setOwner] = useState('');
  const [criteria, setCriteria] = useState(['']);

  const addCriteria = () => setCriteria([...criteria, '']);
  const updateCriteria = (index: number, value: string) => {
    const newCriteria = [...criteria];
    newCriteria[index] = value;
    setCriteria(newCriteria);
  };

  const validCriteria = useMemo(() => criteria.filter((item) => item.trim() !== ''), [criteria]);

  const draftStatus = useMemo<WorkItem['status']>(() => {
    if (validCriteria.length > 0 && owner) return 'Active';
    if (validCriteria.length > 0) return 'Ready';
    return 'Draft';
  }, [owner, validCriteria]);

  const handleSubmit = () => {
    if (!title.trim()) return;

    const newItem: WorkItem = {
      id: `wi-${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
      title: title.trim(),
      goal: goal.trim(),
      priority,
      owner_seat_id: owner || undefined,
      acceptance_criteria: validCriteria,
      status: draftStatus,
      depends_on: [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    addWorkItem(newItem);
    onClose();
  };

  return (
    <div className="flex flex-col h-full bg-background animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="flex items-center justify-between p-4 border-b border-border bg-card">
        <div className="space-y-1">
          <h2 className="text-lg font-bold flex items-center gap-2 text-primary">
            <ListTodo size={20} />
            {t.forms.createWorkItem}
          </h2>
          <p className="text-xs text-muted-foreground">
            用一个可验收、可分配、可交接的结构，把今天的真实协调动作沉淀成项目权威任务，而不是继续停留在聊天或终端滚屏里。
          </p>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        <div className="rounded-xl border border-border bg-secondary/40 p-4 grid grid-cols-1 md:grid-cols-3 gap-3 text-xs">
          <div>
            <div className="text-muted-foreground mb-1">当前状态预览</div>
            <div className="font-semibold text-foreground">{getWorkItemStatusLabel(draftStatus)}</div>
          </div>
          <div>
            <div className="text-muted-foreground mb-1">已填写验收标准</div>
            <div className="font-semibold text-foreground">{validCriteria.length} 条</div>
          </div>
          <div>
            <div className="text-muted-foreground mb-1">当前指派</div>
            <div className="font-semibold text-foreground">{owner ? seats.find((seat) => seat.id === owner)?.name || owner : '暂未指派'}</div>
          </div>
        </div>

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">
            {t.forms.title} <span className="text-status-error">*</span>
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="例如：中文演示数据与真实量级刷新"
            className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary font-medium"
            autoFocus
          />
          <p className="text-[11px] text-muted-foreground">
            标题建议直接写“要交付的结果”，避免只写动作词或模糊备注。
          </p>
        </div>

        <div className="space-y-1.5">
          <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.goal}</label>
          <textarea
            rows={4}
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
            placeholder="请详细说明这项工作要解决什么问题、为什么现在要做、影响哪些界面/席位，以及交付后应当带来什么变化。"
            className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary resize-none"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.priority}</label>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value as WorkItem['priority'])}
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
            >
              {priorityOptions.map((option) => (
                <option key={option} value={option}>
                  {getPriorityLabel(option)}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-1.5">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.assignee}</label>
            <select
              value={owner}
              onChange={(e) => setOwner(e.target.value)}
              className="w-full bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
            >
              <option value="">暂不指派（保持草稿/就绪）</option>
              {seats.map((seat) => (
                <option key={seat.id} value={seat.id}>
                  {seat.name}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between gap-3">
            <label className="text-[10px] font-bold text-muted-foreground uppercase tracking-wider">{t.forms.acceptanceCriteria}</label>
            <span className="text-[11px] text-muted-foreground">至少写出能被别人复核的完成条件</span>
          </div>
          {criteria.map((criterion, index) => (
            <div key={index} className="flex gap-2">
              <input
                type="text"
                value={criterion}
                onChange={(e) => updateCriteria(index, e.target.value)}
                placeholder={`验收标准 ${index + 1}：例如“收件箱、时间线、详情视图的文案全部切换为中文，且无断链”`}
                className="flex-1 bg-secondary border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-1 focus:ring-primary"
              />
              {criteria.length > 1 && (
                <button
                  onClick={() => setCriteria(criteria.filter((_, itemIndex) => itemIndex !== index))}
                  className="text-muted-foreground hover:text-status-error px-1"
                >
                  <X size={16} />
                </button>
              )}
            </div>
          ))}
          <button
            onClick={addCriteria}
            className="flex items-center gap-1.5 text-xs text-primary hover:underline font-bold pt-2 uppercase tracking-widest"
          >
            <Plus size={12} /> {t.forms.addCriteria}
          </button>
        </div>
      </div>

      <div className="p-4 border-t border-border bg-card flex justify-end gap-3">
        <button onClick={onClose} className="px-4 py-2 text-xs font-bold uppercase tracking-widest text-muted-foreground hover:bg-black/5 rounded-lg transition-colors">
          {t.common.cancel}
        </button>
        <button
          onClick={handleSubmit}
          disabled={!title.trim()}
          className="px-6 py-2 text-xs font-bold uppercase tracking-widest bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors shadow-lg shadow-primary/20 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          创建并写入
        </button>
      </div>
    </div>
  );
};

export default WorkItemForm;
