import React, { useMemo, useState } from 'react';
import { Inbox, Search, X } from 'lucide-react';
import MorningDigest from '../components/MorningDigest';
import InboxItem from '../components/InboxItem';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';

interface InboxViewProps {
  onSelectObject: (item: any) => void;
  selectedObjectId?: string | null;
}

const InboxView: React.FC<InboxViewProps> = ({ onSelectObject, selectedObjectId }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const inboxItems = currentData?.inboxItems || [];

  const filteredItems = useMemo(() => {
    if (!searchTerm.trim()) return inboxItems;
    const term = searchTerm.toLowerCase();
    return inboxItems.filter(item => 
      item.summary.toLowerCase().includes(term) ||
      item.object_ref.toLowerCase().includes(term) ||
      item.actor.toLowerCase().includes(term) ||
      item.type.toLowerCase().includes(term) ||
      item.priority.toLowerCase().includes(term)
    );
  }, [inboxItems, searchTerm]);

  return (
    <div className="flex-1 flex flex-col h-full bg-background overflow-hidden">
      <div className="flex-1 overflow-y-auto p-8 space-y-8">
        <MorningDigest />

        <div className="space-y-4">
          <div className="flex items-center justify-between px-2">
            <div className="flex items-center gap-4">
              <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">INBOX</h2>
              <div className="relative group">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-secondary group-focus-within:text-primary transition-colors" />
                <input
                  type="text"
                  placeholder="过滤收件箱..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9 pr-8 py-1.5 bg-secondary/60 border border-border/40 rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all w-48 shadow-sm"
                />
                {searchTerm && (
                  <button 
                    onClick={() => setSearchTerm('')}
                    className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 hover:bg-secondary/80 hover:text-primary rounded-full text-text-secondary transition-colors"
                  >
                    <X size={12} />
                  </button>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-black text-text-secondary uppercase tracking-widest bg-secondary px-2 py-0.5 rounded border border-border/40">
                {searchTerm ? `${filteredItems.length} / ${inboxItems.length}` : inboxItems.length} ITEMS
              </span>
            </div>
          </div>

          {filteredItems.length > 0 ? (
            <div className="divide-y divide-border/60 bg-card rounded-2xl border border-border/60 shadow-sm overflow-hidden">
              {filteredItems.map((item) => (
                <div key={item.id} className="outline-none">
                  <InboxItem
                    item={item}
                    onSelectObject={onSelectObject}
                    onAction={(type, id) => console.log(`Inbox action: ${type} on ${id}`)}
                    isActive={!!(selectedObjectId === item.id || (item.object_ref && selectedObjectId === item.object_ref.toLowerCase()))}
                  />
                </div>
              ))}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center py-20 bg-secondary/30 rounded-2xl border border-dashed border-border/60">
              <div className="p-4 bg-background rounded-full border border-border/40 text-text-secondary mb-4">
                <Inbox className="w-8 h-8 opacity-20" />
              </div>
              <p className="text-sm font-bold text-text-secondary">
                {searchTerm ? '未找到匹配项' : t.inbox.clear}
              </p>
              <p className="text-xs text-text-secondary uppercase tracking-widest mt-1 opacity-60">
                {searchTerm ? '请尝试更换关键词' : t.inbox.clearSub}
              </p>
              {searchTerm && (
                <button 
                  onClick={() => setSearchTerm('')}
                  className="mt-4 text-[10px] font-black text-primary uppercase tracking-widest hover:underline"
                >
                  清除过滤条件
                </button>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default InboxView;
