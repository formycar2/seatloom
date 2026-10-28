import React, { useEffect, useMemo, useState } from 'react';
import { ChevronDown, Eye, EyeOff, Filter, Search } from 'lucide-react';
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

/**
 * Where the operator's event-type visibility choice is remembered.
 *
 * 2026-10-20 (AEGIS-2026-10-20-ui-memory-and-desync-v1 item 2): the choice is
 * persisted per PROJECT, because the timeline is per-project — a global key would
 * carry one project's preference into another project's timeline and hide rows the
 * operator never chose to hide there.
 */
const VISIBILITY_KEY_PREFIX = 'seatloom.timeline.visibility.';

/**
 * Event types collapsed by default, so the ledger's memory traffic stops crowding
 * out the work the operator opens the timeline to see.
 *
 * Measured on live before this was written (2026-10-20): of the newest 400 rows,
 * 106 are MemoryRecalled and 68 are MemoryCreated, plus 6 TopicOpened — 180 of 400,
 * while every dispatch/acceptance/verdict row the operator reads is a minority. The
 * operator's ruling was "初始收起", and the counts are restated on screen next to
 * the control (collapsing is not hiding).
 *
 * A record of the operator's earlier choice wins over this list; the list is only
 * the first-run default.
 */
const COLLAPSED_BY_DEFAULT: ReadonlySet<string> = new Set([
  'MemoryCreated',
  'MemoryRecalled',
  'TopicOpened',
]);

const TimelineView: React.FC<TimelineViewProps> = ({
  onSelectObject,
  onOpenArtifact,
  filters,
  onFiltersChange,
}) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const [searchTerm, setSearchTerm] = useState('');
  const [showFilters, setShowFilters] = useState(false);
  const [typeFilter, setTypeFilter] = useState<string>('all');

  // The event types the operator has collapsed. This replaces `templateFilter`, a
  // `useState` that no filter ever read — a switch that looked live and did
  // nothing, which is worse than no switch because it says "already filtered".
  const [hiddenTypes, setHiddenTypes] = useState<Set<string>>(() => new Set(COLLAPSED_BY_DEFAULT));
  const [visibilityLoaded, setVisibilityLoaded] = useState(false);

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  // Derived BEFORE the early return below, so the hook order is unconditional.
  // Its own useMemo: `currentData?.events || []` is a fresh array each render, which
  // would make every dependent memo below re-run every frame.
  const events = useMemo(() => currentData?.events || [], [currentData]);

  // Read the remembered choice for THIS project once it is known. A missing key,
  // an unreadable one, or a project with no record all fall back to the
  // first-run default rather than to an error: a storage failure must not blank
  // the timeline.
  useEffect(() => {
    setVisibilityLoaded(false);
    if (!activeProjectId) return;
    let remembered: string[] | null = null;
    try {
      const raw = localStorage.getItem(VISIBILITY_KEY_PREFIX + activeProjectId);
      if (raw) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.every((x) => typeof x === 'string')) {
          remembered = parsed;
        }
      }
    } catch {
      // Unreadable or corrupt — fall through to the default below.
    }
    setHiddenTypes(remembered ? new Set(remembered) : new Set(COLLAPSED_BY_DEFAULT));
    setVisibilityLoaded(true);
  }, [activeProjectId]);

  const toggleTypeVisibility = (type: string) => {
    setHiddenTypes((prev) => {
      const next = new Set(prev);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      try {
        if (activeProjectId) {
          localStorage.setItem(VISIBILITY_KEY_PREFIX + activeProjectId, JSON.stringify([...next]));
        }
      } catch {
        // A read-only or full storage must not break the toggle; the choice simply
        // is not remembered this time.
      }
      return next;
    });
  };

  const showAllTypes = () => {
    setHiddenTypes(new Set());
    try {
      if (activeProjectId) {
        localStorage.setItem(VISIBILITY_KEY_PREFIX + activeProjectId, JSON.stringify([]));
      }
    } catch {
      /* not remembered; harmless */
    }
  };

  const eventTypes = useMemo(() => Array.from(new Set(events.map((e) => e.event_type))), [events]);

  const filteredEvents = useMemo(() => {
    return events.filter((event) => {
      const matchesSearch =
        searchTerm === '' ||
        event.payload?.title?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.payload?.summary?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.event_type.toLowerCase().includes(searchTerm.toLowerCase());

      const matchesType = typeFilter === 'all' || event.event_type === typeFilter;

      // The real filter this view lacked.
      const matchesVisibility = !hiddenTypes.has(event.event_type);

      return matchesSearch && matchesType && matchesVisibility;
    });
  }, [events, searchTerm, typeFilter, hiddenTypes]);

  // What collapsing is hiding, in numbers the operator can check.
  const collapsedSummary = useMemo(() => {
    const present = eventTypes.filter((type) => hiddenTypes.has(type));
    const hiddenCount = events.filter((e) => hiddenTypes.has(e.event_type)).length;
    return { types: present, hiddenCount };
  }, [eventTypes, events, hiddenTypes]);

  // 2026-10-17 (Mr. Zhang's bug: switching projects filtered nothing): with no
  // project resolved yet this says so, rather than rendering an empty list that
  // reads as "this project has nothing". SeatsView already had this guard; these
  // two rendered a silent empty.
  if (!currentData) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)]">
        <p className="text-sm">请选择一个项目</p>
      </div>
    );
  }

  const clearFilters = () => {
    setSearchTerm('');
    setTypeFilter('all');
    showAllTypes();
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-background overflow-hidden">
      <div className="p-8 border-b border-border/40 space-y-6">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">
            TIMELINE
          </h2>
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

        {/* 2026-10-20: collapsing is not hiding. This strip names which types are
            collapsed, how many rows that removes, and gives one click to show them
            again. Without it, a shorter list reads as "nothing happened lately". */}
        {visibilityLoaded && collapsedSummary.types.length > 0 && (
          <div className="flex items-center gap-3 flex-wrap px-4 py-2 bg-secondary/40 border border-border/40 rounded-xl">
            <EyeOff size={14} className="text-text-secondary shrink-0" />
            <span className="text-xs font-bold text-text-secondary">
              已收起 {collapsedSummary.types.length} 类共 {collapsedSummary.hiddenCount} 条:
            </span>
            <div className="flex gap-1.5 flex-wrap">
              {collapsedSummary.types.map((type) => (
                <button
                  key={type}
                  onClick={() => toggleTypeVisibility(type)}
                  title={`显示 ${type}`}
                  className="px-2 py-0.5 rounded-md bg-card border border-border/60 text-[10px] font-black text-text-secondary hover:text-primary hover:border-primary/40 transition-all flex items-center gap-1"
                >
                  <Eye size={10} />
                  {type}
                </button>
              ))}
            </div>
            <button
              onClick={showAllTypes}
              className="ml-auto text-[10px] font-black text-primary uppercase tracking-widest hover:underline"
            >
              全部展开
            </button>
          </div>
        )}

        {showFilters && (
          <div className="flex flex-wrap gap-4 items-end animate-in fade-in slide-in-from-top-2 duration-200">
            <div className="space-y-1.5">
              <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">
                事件类型
              </label>
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

            <div className="space-y-1.5">
              <label className="text-[10px] font-black text-text-secondary uppercase tracking-widest ml-1">
                显示/收起类型
              </label>
              <div className="flex gap-1.5 flex-wrap max-w-md">
                {eventTypes.map((type) => {
                  const hidden = hiddenTypes.has(type);
                  return (
                    <button
                      key={type}
                      type="button"
                      onClick={() => toggleTypeVisibility(type)}
                      title={hidden ? `显示 ${type}` : `收起 ${type}`}
                      className={`px-2 py-1 rounded-lg border text-[10px] font-bold transition-all flex items-center gap-1 ${
                        hidden
                          ? 'bg-secondary/60 border-border/40 text-text-secondary'
                          : 'bg-primary/10 border-primary/40 text-primary'
                      }`}
                    >
                      {hidden ? <EyeOff size={10} /> : <Eye size={10} />}
                      {type}
                    </button>
                  );
                })}
              </div>
            </div>

            <button
              onClick={clearFilters}
              className="px-4 py-1.5 text-[10px] font-black text-text-secondary uppercase tracking-widest hover:text-primary transition-colors mb-0.5"
            >
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
            <p className="text-xs text-text-secondary uppercase tracking-widest mt-1 opacity-60">
              请尝试调整搜索词或筛选条件
            </p>
            <button
              onClick={clearFilters}
              className="text-xs font-black text-primary uppercase tracking-widest mt-4 hover:underline"
            >
              重置筛选条件
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default TimelineView;
