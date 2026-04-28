import React, { useMemo, useState } from 'react';
import { ChevronRight, Circle, Plus, Search } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';
import { WorkItem } from '../types';
import { getPriorityLabel, getWorkItemStatusLabel } from '../utils/display';

interface WorkItemsViewProps {
  onSelectWI: (wi: WorkItem) => void;
}

const WorkItemsView: React.FC<WorkItemsViewProps> = ({ onSelectWI }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const workItems = currentData?.workItems || [];
  const seats = currentData?.seats || [];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active':
        return 'text-status-active';
      case 'Blocked':
        return 'text-status-error';
      case 'Done':
        return 'text-status-done';
      case 'Drifted':
        return 'text-status-drifted';
      case 'InReview':
        return 'text-primary';
      default:
        return 'text-muted-foreground';
    }
  };

  const filteredItems = useMemo(
    () =>
      workItems.filter(
        (workItem) =>
          workItem.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
          workItem.id.toLowerCase().includes(searchTerm.toLowerCase()) ||
          (workItem.goal || '').toLowerCase().includes(searchTerm.toLowerCase()),
      ),
    [workItems, searchTerm],
  );

  const criticalCount = filteredItems.filter((workItem) => workItem.priority === 'Critical' || workItem.priority === 'High').length;

  return (
    <div className="flex-1 flex flex-col overflow-hidden bg-background animate-in fade-in duration-500">
      <div className="px-6 py-4 border-b border-border flex items-center justify-between bg-card/50">
        <div>
          <h2 className="text-base font-bold text-foreground">{t.sidebar.workitems}</h2>
          <p className="text-xs text-muted-foreground mt-1">当前视图聚合了负责席位、优先级、验收标准密度和依赖压力。</p>
        </div>
        <button onClick={() => onSelectWI('Action:AddWorkItem' as any)} className="flex items-center gap-2 px-3 py-1.5 bg-primary text-primary-foreground text-xs font-semibold rounded-lg hover:opacity-90 transition-all shadow-lg shadow-primary/20">
          <Plus size={14} />
          {t.forms.createWorkItem}
        </button>
      </div>

      <div className="px-4 py-2 border-b border-border bg-secondary/30 flex items-center gap-4 flex-wrap">
        <div className="relative flex-1 min-w-[240px] max-w-sm">
          <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            placeholder="按工作项 ID、标题或目标描述搜索"
            value={searchTerm}
            onChange={(event) => setSearchTerm(event.target.value)}
            className="w-full bg-background border border-border rounded-lg pl-9 pr-3 py-1.5 text-xs focus:outline-none focus:ring-1 focus:ring-primary shadow-sm"
          />
        </div>
        <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
          <span className="px-2 py-1 rounded-full bg-background border border-border">共 {filteredItems.length} 项</span>
          <span className="px-2 py-1 rounded-full bg-background border border-border">高优先级 {criticalCount} 项</span>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="space-y-6">
          <div>
            <div className="flex items-center gap-2 mb-3 text-xs font-semibold text-muted-foreground tracking-wide">
              <ChevronRight size={14} />
              当前工作面（{filteredItems.length}）
            </div>
            <div className="grid grid-cols-1 gap-3">
              {filteredItems.map((workItem) => {
                const owner = seats.find((seat) => seat.id === workItem.owner_seat_id);

                return (
                  <div
                    key={workItem.id}
                    onClick={() => onSelectWI(workItem)}
                    className="sl-clickable p-4 border border-border bg-card flex items-start justify-between group shadow-sm"
                    tabIndex={0}
                  >
                    <div className="flex-1 min-w-0 pr-4">
                      <div className="flex items-center gap-3 mb-2 flex-wrap">
                        <Circle size={8} className={`fill-current ${getStatusColor(workItem.status)}`} />
                        <span className="monospace text-[11px] font-semibold text-muted-foreground bg-secondary px-1.5 rounded">{workItem.id}</span>
                        <span className={`text-[11px] font-semibold ${workItem.priority === 'High' || workItem.priority === 'Critical' ? 'text-status-error' : 'text-status-warning'}`}>
                          {getPriorityLabel(workItem.priority)}
                        </span>
                        <span className="text-[11px] text-muted-foreground">{getWorkItemStatusLabel(workItem.status)}</span>
                      </div>
                      <div className="text-sm font-bold leading-snug group-hover:text-primary transition-colors text-foreground">{workItem.title}</div>
                      {workItem.goal && <div className="text-[12px] text-muted-foreground mt-2 leading-6 whitespace-pre-wrap">{workItem.goal}</div>}
                      <div className="mt-3 flex items-center gap-3 flex-wrap text-[11px] text-muted-foreground">
                        <span>负责人：{owner?.name || '未指定'}</span>
                        <span>验收项：{workItem.acceptance_criteria.length}</span>
                        <span>依赖：{workItem.depends_on.length}</span>
                      </div>
                    </div>
                    <div className="flex flex-col items-end gap-2 text-right min-w-[120px]">
                      <div className="text-xs font-medium text-muted-foreground opacity-80">更新于 {new Date(workItem.updated_at).toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })}</div>
                      <div className="text-[11px] text-primary font-semibold">查看详情</div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WorkItemsView;
