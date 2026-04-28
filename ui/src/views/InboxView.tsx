import React from 'react';
import MorningDigest from '../components/MorningDigest';
import InboxItem from '../components/InboxItem';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';
import { InboxItem as InboxItemType } from '../types';

interface InboxViewProps {
  onSelectObject: (item: InboxItemType) => void;
}

const InboxView: React.FC<InboxViewProps> = ({ onSelectObject }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData, removeInboxItem } = useDataStore();

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const inboxItems = currentData?.inboxItems || [];

  const handleAction = (id: string, action: string) => {
    console.log(`Action: ${action} on ${id}`);
    removeInboxItem(id);
  };

  const sortedItems = [...inboxItems].sort((a, b) => {
    const priorityMap: Record<string, number> = { Critical: 3, Normal: 2, Low: 1 };
    return (priorityMap[b.priority] || 0) - (priorityMap[a.priority] || 0);
  });

  const criticalCount = sortedItems.filter((item) => item.priority === 'Critical').length;
  const normalCount = sortedItems.filter((item) => item.priority === 'Normal').length;
  const lowCount = sortedItems.filter((item) => item.priority === 'Low').length;

  return (
    <div className="flex-1 overflow-y-auto p-6 space-y-6 animate-in fade-in duration-500">
      <MorningDigest />

      <div className="space-y-3">
        <div className="flex items-center justify-between gap-3 flex-wrap">
          <h2 className="text-base font-bold flex items-center gap-2 text-foreground">
            {t.inbox.title}
            <span className="text-primary bg-primary/10 px-2 py-0.5 rounded-full text-xs font-semibold">共 {sortedItems.length} 项待处理</span>
          </h2>
          <div className="flex items-center gap-2 flex-wrap text-[11px]">
            <span className="px-2 py-1 rounded-full border border-status-error/20 bg-status-error/10 text-status-error">紧急 {criticalCount}</span>
            <span className="px-2 py-1 rounded-full border border-border bg-card text-muted-foreground">常规 {normalCount}</span>
            <span className="px-2 py-1 rounded-full border border-border bg-card text-muted-foreground">低优先级 {lowCount}</span>
          </div>
        </div>
        <p className="text-sm text-muted-foreground leading-relaxed">
          这里汇总了今日日志归一化后的交接、验收、会话输入请求与漂移回收结果。请按优先级处理，不要再依赖零散终端窗口去回忆待办。
        </p>
      </div>

      <div className="border border-border rounded-xl overflow-hidden divide-y divide-border bg-card shadow-sm">
        {sortedItems.length > 0 ? (
          sortedItems.map((item) => (
            <div key={item.id} className="sl-clickable focus-within:bg-black/5 outline-none" onClick={() => onSelectObject(item)}>
              <InboxItem item={item} onAction={handleAction} />
            </div>
          ))
        ) : (
          <div className="p-12 text-center text-muted-foreground flex flex-col items-center space-y-2">
            <div className="w-12 h-12 bg-status-active/10 text-status-active rounded-full flex items-center justify-center">
              <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <p className="text-sm font-bold">{t.inbox.clear}</p>
            <p className="text-xs text-muted-foreground">{t.inbox.clearSub}</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default InboxView;
