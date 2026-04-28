import React from 'react';
import { InboxItem as InboxItemType } from '../types';
import { getInboxPriorityLabel } from '../utils/display';

interface InboxItemProps {
  item: InboxItemType;
  onAction: (id: string, action: string) => void;
}

const InboxItem: React.FC<InboxItemProps> = ({ item, onAction }) => {
  const priorityStyles: Record<string, string> = {
    Critical: 'bg-destructive/10 text-destructive border-destructive/20',
    Normal: 'bg-status-warning/10 text-status-warning border-status-warning/20',
    Low: 'bg-secondary text-muted-foreground border-border',
  };

  const isHandoff = item.type.includes('交接');

  return (
    <div className="p-4 hover:bg-black/5 cursor-pointer group flex items-start gap-4 transition-colors focus:outline-none focus:bg-black/5" tabIndex={0}>
      <div className={`mt-0.5 px-2 py-1 rounded-full border text-[10px] font-bold whitespace-nowrap ${priorityStyles[item.priority] || priorityStyles.Low}`}>
        {getInboxPriorityLabel(item.priority)}
      </div>
      <div className="flex-1 min-w-0 space-y-1.5">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-[11px] font-semibold text-primary">{item.type}</span>
          <span className="text-[11px] text-muted-foreground font-medium">{item.actor}</span>
          <span className="monospace text-[11px] bg-secondary px-1.5 py-0.5 rounded border border-border text-muted-foreground ml-auto">
            {item.object_ref}
          </span>
        </div>
        <div className="text-sm font-semibold text-foreground leading-6 whitespace-pre-wrap">{item.summary}</div>
      </div>
      <div className="flex flex-col items-end gap-2">
        <div className="text-xs text-muted-foreground whitespace-nowrap monospace">{item.timestamp}</div>
        <div className="flex items-center gap-1 transition-opacity">
          {isHandoff ? (
            <>
              <button
                onClick={(event) => {
                  event.stopPropagation();
                  onAction(item.id, 'accept');
                }}
                className="px-2 py-1 text-[11px] bg-status-active/10 text-status-active hover:bg-status-active hover:text-white rounded border border-status-active/20 font-semibold transition-colors focus:outline-none focus:ring-1 focus:ring-status-active"
              >
                接收
              </button>
              <button
                onClick={(event) => {
                  event.stopPropagation();
                  onAction(item.id, 'return');
                }}
                className="px-2 py-1 text-[11px] bg-destructive/10 text-destructive hover:bg-destructive hover:text-white rounded border border-destructive/20 font-semibold transition-colors focus:outline-none focus:ring-1 focus:ring-destructive"
              >
                退回
              </button>
            </>
          ) : (
            <button
              onClick={(event) => {
                event.stopPropagation();
                onAction(item.id, 'resolve');
              }}
              className="px-2 py-1 text-[11px] bg-secondary border border-border hover:bg-black/10 text-foreground rounded font-semibold transition-colors focus:outline-none focus:ring-1 focus:ring-primary"
            >
              标记已读
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default InboxItem;
