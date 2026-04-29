import React, { useMemo, useState } from 'react';
import { CheckCircle2, ChevronRight, Clock, Search, X } from 'lucide-react';
import { WorkItem } from '../types';
import { useDataStore } from '../stores/useDataStore';
import { formatDateTimeZh, getPriorityLabel, getWorkItemStatusLabel } from '../utils/display';

interface WorkItemsViewProps {
  onSelectWI: (object: any) => void;
}

const WorkItemsView: React.FC<WorkItemsViewProps> = ({ onSelectWI }) => {
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const workItems = currentData?.workItems || [];

  const filteredItems = useMemo(() => {
    if (!searchTerm.trim()) return workItems;
    const term = searchTerm.toLowerCase();
    return workItems.filter(item => 
      item.id.toLowerCase().includes(term) ||
      item.title.toLowerCase().includes(term) ||
      (item.goal && item.goal.toLowerCase().includes(term))
    );
  }, [workItems, searchTerm]);

  return (
    <div className="flex-1 overflow-y-auto p-10 bg-background">
      <div className="max-w-5xl mx-auto space-y-10">
        <header className="space-y-4 border-b border-border/40 pb-8">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <h1 className="text-3xl font-black tracking-tight text-text-primary uppercase">WORK ITEMS</h1>
              <p className="text-text-secondary font-bold opacity-60 uppercase tracking-widest text-xs">Active Task Ledger · 活跃工作项账本</p>
            </div>
            
            <div className="relative group">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary group-focus-within:text-primary transition-colors" />
              <input
                type="text"
                placeholder="搜索 ID 或 任务标题..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-10 pr-10 py-2.5 bg-secondary/60 border border-border/40 rounded-2xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all w-64 shadow-sm"
              />
              {searchTerm && (
                <button 
                  onClick={() => setSearchTerm('')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 p-1 hover:bg-accent rounded-full text-text-secondary"
                >
                  <X size={14} />
                </button>
              )}
            </div>
          </div>
          
          <div className="flex items-center gap-2">
            <span className="text-[10px] font-black text-text-muted uppercase tracking-widest bg-secondary px-2 py-0.5 rounded border border-border/40">
              {searchTerm ? `${filteredItems.length} / ${workItems.length}` : workItems.length} ACTIVE ITEMS
            </span>
          </div>
        </header>

        {filteredItems.length > 0 ? (
          <div className="grid grid-cols-1 gap-6">
            {filteredItems.map((item) => (
              <div
                key={item.id}
                onClick={() => onSelectWI(item)}
                className="group bg-card border border-border/60 p-6 rounded-2xl cursor-pointer hover:border-primary/40 hover:shadow-xl hover:shadow-primary/5 transition-all flex flex-col md:flex-row md:items-center gap-6"
              >
                <div className="flex-1 space-y-4">
                  <div className="flex items-center gap-3">
                    <span className="monospace text-xs font-black text-primary bg-accent px-2 py-0.5 rounded border border-primary/10 shadow-sm">{item.id.toUpperCase()}</span>
                    <div className="flex items-center gap-2">
                      <span className={`text-[10px] font-black uppercase tracking-widest px-2 py-0.5 rounded border ${
                        item.priority === 'Critical' ? 'bg-status-error/10 text-status-error border-status-error/20' :
                        item.priority === 'High' ? 'bg-status-warning/10 text-status-warning border-status-warning/20' :
                        'bg-secondary text-text-secondary border-border/40'
                      }`}>
                        {item.priority}
                      </span>
                      <span className="text-[10px] font-bold text-text-secondary uppercase tracking-tighter bg-secondary/60 px-2 py-0.5 rounded border border-border/40">
                        {getWorkItemStatusLabel(item.status)}
                      </span>
                    </div>
                  </div>
                  <h3 className="text-xl font-bold text-text-primary leading-tight group-hover:text-primary transition-colors">{item.title}</h3>
                  <p className="text-sm text-text-secondary leading-relaxed line-clamp-2 opacity-80">{item.goal}</p>
                </div>

                <div className="flex flex-row md:flex-col items-center md:items-end gap-4 md:gap-2 text-[10px] font-bold text-text-secondary opacity-60 whitespace-nowrap">
                  <div className="flex items-center gap-1.5 uppercase tracking-tighter">
                    <Clock size={12} />
                    更新: {formatDateTimeZh(item.updated_at)}
                  </div>
                  <div className="flex items-center gap-1.5 uppercase tracking-tighter">
                    <CheckCircle2 size={12} />
                    验收标准: {item.acceptance_criteria.length} 项
                  </div>
                  <div className="mt-auto hidden md:block">
                    <ChevronRight size={16} className="text-primary opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" />
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-32 text-center bg-secondary/20 rounded-3xl border border-dashed border-border/60">
            <div className="p-5 bg-background rounded-full border border-border/40 text-text-secondary mb-6 opacity-20">
              <Search className="w-10 h-10" />
            </div>
            <p className="text-lg font-bold text-text-secondary">未找到匹配的工作项</p>
            <p className="text-sm text-text-secondary opacity-60 mt-1">请尝试更换搜索词，或清除过滤条件以查看全部。</p>
            <button 
              onClick={() => setSearchTerm('')}
              className="mt-6 px-6 py-2 bg-primary text-white rounded-xl text-xs font-black uppercase tracking-widest hover:opacity-90 transition-all shadow-lg shadow-primary/20"
            >
              显示全部工作项
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkItemsView;
