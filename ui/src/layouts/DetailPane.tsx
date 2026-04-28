import React from 'react';
import { ExternalLink, Shield, X } from 'lucide-react';
import { getObjectTypeLabel } from '../utils/display';

interface DetailPaneProps {
  title: string;
  type: string;
  id: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
  onClose: () => void;
}

const DetailPane: React.FC<DetailPaneProps> = ({ title, type, id, children, actions, onClose }) => {
  return (
    <div className="flex flex-col h-full bg-background border-l border-border shadow-2xl">
      <div className="flex items-center justify-between p-4 border-b border-border bg-card">
        <div className="flex items-center gap-3 min-w-0">
          <button onClick={onClose} className="p-1.5 hover:bg-accent hover:text-primary rounded-md transition-all text-muted-foreground">
            <X size={16} />
          </button>
          <div className="min-w-0">
            <div className="flex items-center gap-1.5 mb-0.5">
              <Shield size={10} className="text-primary" />
              <div className="text-[9px] font-black text-muted-foreground tracking-[0.2em]">{getObjectTypeLabel(type)}</div>
            </div>
            <div className="text-sm font-bold text-foreground truncate">{title}</div>
            {id && <div className="text-[11px] font-medium monospace text-muted-foreground mt-1">{id}</div>}
          </div>
        </div>
        <button className="p-1.5 hover:bg-accent hover:text-primary rounded-md transition-all text-primary" title="引用来源">
          <ExternalLink size={16} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-5 space-y-6">{children}</div>

      {actions && <div className="p-4 border-t border-border bg-card grid grid-cols-2 gap-3">{actions}</div>}
    </div>
  );
};

export default DetailPane;
