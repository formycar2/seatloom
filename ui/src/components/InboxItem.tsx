import React from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Clock,
  MessageSquare,
  Receipt,
  RotateCcw,
} from 'lucide-react';
import { InboxItem as InboxItemType } from '../types';

interface InboxItemProps {
  item: InboxItemType;
  onSelectObject: (item: InboxItemType) => void;
  onAction: (type: string, id: string) => void;
  isActive?: boolean;
}

const InboxItem: React.FC<InboxItemProps> = ({ item, onSelectObject, onAction, isActive }) => {
  const getIcon = () => {
    switch (item.type) {
      case '待处理交接': return <Receipt size={18} />;
      case '验收待决策': return <AlertCircle size={18} />;
      case '输入请求': return <MessageSquare size={18} />;
      case '背景记录': return <Clock size={18} />;
      case '会话恢复': return <RotateCcw size={18} />;
      default: return <AlertTriangle size={18} />;
    }
  };

  const priorityColor =
    (item.priority as string) === 'Critical'
      ? isActive ? 'bg-surface/20 text-surface border-surface/30' : 'bg-status-error/10 text-status-error border-status-error/20'
      : (item.priority as string) === 'High'
        ? isActive ? 'bg-surface/20 text-surface border-surface/30' : 'bg-status-warning/10 text-status-warning border-status-warning/20'
        : isActive ? 'bg-surface/10 text-surface/70 border-surface/20' : 'bg-secondary text-text-secondary border-border/40';

  const icon = getIcon();

  return (
    <div 
      className={`p-4 cursor-pointer group flex items-start gap-4 transition-all focus:outline-none ${isActive ? 'bg-primary text-surface shadow-lg' : 'hover:bg-primary/5'}`} 
      tabIndex={0} 
      onClick={() => onSelectObject(item)}
    >
      <div className={`mt-1 p-2 rounded-lg shadow-sm transition-colors border ${isActive ? 'bg-surface/10 text-surface border-surface/20' : 'bg-secondary text-text-secondary border-transparent group-hover:bg-card group-hover:border-primary/10 group-hover:text-primary'}`}>
        {icon}
      </div>
      <div className="flex-1 min-w-0 space-y-1">
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-2 min-w-0">
            <span className={`text-[10px] font-black uppercase tracking-widest px-1.5 py-0.5 rounded border ${priorityColor}`}>
              {item.priority}
            </span>
            <span className={`text-[10px] font-bold uppercase tracking-tighter px-1.5 py-0.5 rounded border ${isActive ? 'bg-surface/10 text-surface/90 border-surface/20' : 'bg-secondary/60 text-text-secondary border-border/40'}`}>
              {item.type}
            </span>
          </div>
          <span className={`text-[10px] font-bold whitespace-nowrap ${isActive ? 'text-surface/60' : 'text-text-muted opacity-60'}`}>{item.timestamp}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className={`text-xs font-bold opacity-60 uppercase tracking-tighter monospace ${isActive ? 'text-surface' : 'text-text-secondary'}`}>{item.actor}</span>
          <span className={`text-xs ${isActive ? 'text-surface/40' : 'text-text-muted opacity-40'}`}>·</span>
          <span className={`text-xs font-black uppercase tracking-tighter monospace ${isActive ? 'text-surface' : 'text-primary'}`}>{item.object_ref}</span>
        </div>
        <p className={`text-sm font-bold leading-relaxed transition-colors ${isActive ? 'text-surface' : 'text-text-primary group-hover:text-primary'}`}>{item.summary}</p>
        
        {item.linked_artifact_ids && item.linked_artifact_ids.length > 0 && (
          <div className="flex flex-wrap items-center gap-2 mt-3">
            {item.linked_artifact_ids.map(id => (
              <span key={id} className={`text-[9px] font-black uppercase tracking-widest border px-2 py-0.5 rounded shadow-sm transition-all ${isActive ? 'bg-surface/10 border-surface/20 text-surface hover:bg-surface/20' : 'bg-card border-border/60 text-text-secondary hover:border-primary/40 hover:text-primary'}`}>
                {id}
              </span>
            ))}
          </div>
        )}

        <div className={`flex items-center gap-2 mt-4 transition-opacity ${isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`}>
          {item.type === '待处理交接' && (
            <>
              <button 
                onClick={(e) => { e.stopPropagation(); onAction('Accept', item.id); }}
                className={`px-2 py-1 text-[10px] rounded border font-black tracking-widest transition-colors focus:outline-none focus:ring-1 shadow-sm ${isActive ? 'bg-surface text-primary border-surface/20 hover:bg-surface/90' : 'bg-status-active/10 text-status-active hover:bg-status-active hover:text-white border-status-active/20 focus:ring-status-active'}`}
              >
                确认收件
              </button>
              <button 
                onClick={(e) => { e.stopPropagation(); onAction('Reject', item.id); }}
                className={`px-2 py-1 text-[10px] rounded border font-black tracking-widest transition-colors focus:outline-none focus:ring-1 shadow-sm ${isActive ? 'bg-primary-hover text-surface border-surface/20 hover:bg-primary-hover/80' : 'bg-status-error/10 text-status-error hover:bg-status-error hover:text-white border-status-error/20 focus:ring-status-error'}`}
              >
                退回修正
              </button>
            </>
          )}
          <button 
            onClick={(e) => { e.stopPropagation(); onAction('Details', item.id); }}
            className={`px-2 py-1 text-[10px] rounded font-black tracking-widest transition-colors focus:outline-none focus:ring-1 shadow-sm border ${isActive ? 'bg-surface/10 border-surface/20 text-surface hover:bg-surface/20' : 'bg-secondary border-border/60 hover:bg-card text-text-secondary hover:text-primary focus:ring-primary'}`}
          >
            打开详情
          </button>
        </div>
      </div>
    </div>
  );
};

export default InboxItem;
