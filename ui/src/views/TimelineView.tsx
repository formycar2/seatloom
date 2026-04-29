import React, { useMemo, useState } from 'react';
import { ChevronDown, Filter, Search } from 'lucide-react';
import EventRow from '../components/EventRow';
import { useDataStore } from '../stores/useDataStore';
import { ArtifactSubtype, ArtifactTemplate, CanonicalEvent } from '../types';
import { useLocaleStore } from '../stores/useLocaleStore';

interface TimelineViewProps {
  onSelectObject: (type: string, object: any) => void;
  onOpenArtifact: (artifactId: string) => void;
  filters: any;
  onFiltersChange: (filters: any) => void;
}

const TimelineView: React.FC<TimelineViewProps> = ({ onSelectObject, onOpenArtifact, filters, onFiltersChange }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [typeFilter, setTypeFilter] = useState<string>('all');
  const [templateFilter, setTemplateFilter] = useState<string>('all');

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const events = currentData?.events || [];

  const filteredEvents = useMemo(() => {
    return events.filter((event) => {
      const matchesSearch =
        searchTerm === '' ||
        event.payload?.title?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.payload?.summary?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.event_type.toLowerCase().includes(searchTerm.toLowerCase());

      const matchesType = typeFilter === 'all' || event.event_type === typeFilter;

      return matchesSearch && matchesType;
    });
  }, [events, searchTerm, typeFilter]);

  const eventTypes = Array.from(new Set(events.map((e) => e.event_type)));

  const clearFilters = () => {
    setSearchTerm('');
    setTypeFilter('all');
    setTemplateFilter('all');
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-background overflow-hidden">
      <div className="p-8 border-b border-border/40 space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">TIMELINE</h2>
          <div className="flex items-center gap-3">
            <div className="relative group">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-secondary group-focus-within:text-primary transition-colors" />
              <input
                type="text"
                placeholder="搜索事件或证据..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-9 pr-4 py-2 bg-secondary/60 border border-border/40 rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all w-64 shadow-sm"
              />
            </div>
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={`p-2 rounded-xl border transition-all shadow-sm ${showFilters ? 'bg-primary text-white border-primary' : 'bg-card border-border/60 text-text-secondary hover:bg-accent'}`}
            >
              <Filter size={16} />
            </button>
          </div>
        </div>

        {showFilters && (
          <div className="flex flex-wrap gap-4 items-end animate-in fade-in slide-in-from-top-2 duration-200">
            <div className="space-y-1.5">
              <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">事件类型</label>
              <div className="relative">
                <select
                  value={typeFilter}
                  onChange={(e) => setTypeFilter(e.target.value)}
                  className="appearance-none pl-3 pr-8 py-1.5 bg-secondary/60 border border-border/40 rounded-lg text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-white transition-all min-w-[140px]"
                >
                  <option value="all">所有类型</option>
                  {eventTypes.map((type) => (
                    <option key={type} value={type}>
                      {type}
                    </option>
                  ))}
                </select>
                <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-secondary pointer-events-none" />
              </div>
            </div>

            <button onClick={clearFilters} className="px-4 py-1.5 text-[10px] font-black text-text-secondary uppercase tracking-widest hover:text-primary transition-colors mb-0.5">
              重置筛选
            </button>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto px-8 py-4">
        {filteredEvents.length > 0 ? (
          <div className="space-y-4 max-w-4xl">
            {filteredEvents.map((event) => (
              <EventRow
                key={event.event_id}
                event={event}
                onSelectObject={onSelectObject}
                onOpenArtifact={onOpenArtifact}
                onOpenPrimary={() => onSelectObject('Event', event)}
              />
            ))}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-20 text-center">
            <div className="p-4 bg-secondary rounded-full border border-border/40 text-text-secondary mb-4 opacity-20">
              <Search className="w-8 h-8" />
            </div>
            <p className="text-sm font-bold text-text-secondary">没有找到匹配的事件</p>
            <p className="text-xs text-text-secondary uppercase tracking-widest mt-1 opacity-60">请尝试调整搜索词或筛选条件</p>
            <button onClick={clearFilters} className="text-xs font-black text-primary uppercase tracking-widest mt-4 hover:underline">
              重置筛选条件
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default TimelineView;
