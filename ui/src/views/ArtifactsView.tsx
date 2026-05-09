import React, { useMemo, useState } from 'react';
import { Package, Search, FileText, ChevronRight, RefreshCw } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { Artifact, ArtifactTemplate } from '../types';
import { getArtifactTemplateLabel, getArtifactSubtypeLabel } from '../utils/artifacts';
import { api, isTauri } from '../lib/api';
import type { ReconcileResultDto } from '../lib/types-dto';

interface ArtifactsViewProps {
  onSelectArtifact: (artifact: Artifact) => void;
  selectedId?: string | null;
}

const templateColors: Record<string, string> = {
  T1: 'bg-primary/10 text-primary',
  T2: 'bg-[var(--sl-drift-light)] text-[var(--sl-drift)]',
  T3: 'bg-[var(--sl-warning-light)] text-[var(--sl-warning)]',
  T4: 'bg-[var(--sl-review-light)] text-[var(--sl-review)]',
  T5: 'bg-[var(--sl-success-light)] text-[var(--sl-success)]',
  T6: 'bg-[var(--sl-done-light)] text-[var(--sl-done)]',
  T7: 'bg-[var(--sl-error-light)] text-[var(--sl-error)]',
};

const ArtifactsView: React.FC<ArtifactsViewProps> = ({ onSelectArtifact, selectedId }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const [searchTerm, setSearchTerm] = useState('');
  const [templateFilter, setTemplateFilter] = useState<ArtifactTemplate | 'all'>('all');
  const [reconciling, setReconciling] = useState(false);
  const [reconcileResult, setReconcileResult] = useState<ReconcileResultDto | null>(null);
  const [reconcileError, setReconcileError] = useState<string | null>(null);

  const handleReconcile = async () => {
    if (!isTauri()) return;
    setReconciling(true);
    setReconcileResult(null);
    setReconcileError(null);
    try {
      const result = await api.reconcile();
      setReconcileResult(result);
    } catch (e) {
      setReconcileError(String(e));
    } finally {
      setReconciling(false);
    }
  };

  const artifacts = currentData?.artifacts || [];

  const filteredArtifacts = useMemo(() => {
    let result = artifacts;
    if (templateFilter !== 'all') {
      result = result.filter((a) => a.template === templateFilter);
    }
    if (searchTerm.trim()) {
      const term = searchTerm.toLowerCase();
      result = result.filter(
        (a) =>
          a.title.toLowerCase().includes(term) ||
          a.id.toLowerCase().includes(term) ||
          (a.subtype && a.subtype.toLowerCase().includes(term))
      );
    }
    return result;
  }, [artifacts, templateFilter, searchTerm]);

  const templates: ArtifactTemplate[] = ['T1', 'T2', 'T3', 'T4', 'T5', 'T6', 'T7'];

  if (!currentData) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)]">
        <p className="text-sm">请选择一个项目</p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {/* Header with search and filters */}
      <div className="sticky top-0 z-10 bg-canvas border-b border-[var(--sl-border)] px-4 py-3 space-y-2">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-semibold text-[var(--sl-ink)]">产出物</h2>
          <div className="relative flex-1 max-w-xs">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-[var(--sl-ink-muted)]" />
            <input
              type="text"
              placeholder="搜索..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-8 pr-3 py-1.5 text-xs bg-[var(--sl-panel)] border border-[var(--sl-border)] rounded-md focus:outline-none focus:ring-1 focus:ring-primary"
            />
          </div>
          {isTauri() && (
            <button
              onClick={handleReconcile}
              disabled={reconciling}
              className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs bg-[var(--sl-panel)] border border-[var(--sl-border)] rounded-md hover:bg-[var(--sl-hover)] disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <RefreshCw size={12} className={reconciling ? 'animate-spin' : ''} />
              {reconciling ? '正在同步…' : '同步文档'}
            </button>
          )}
        </div>
        {reconcileResult && (
          <div className="text-xs text-[var(--sl-ink-secondary)] bg-[var(--sl-panel)] border border-[var(--sl-border)] rounded px-3 py-1.5">
            已扫描 {reconcileResult.scanned} · 新增 {reconcileResult.inserted} · 更新 {reconcileResult.updated} · 未变 {reconcileResult.unchanged} · 失败 {reconcileResult.failed} · 冲突 {reconcileResult.conflicted}
          </div>
        )}
        {reconcileError && (
          <div className="text-xs text-[var(--sl-error)] bg-[var(--sl-error-light)] border border-[var(--sl-error)] rounded px-3 py-1.5">
            {reconcileError}
          </div>
        )}

        {/* Template filter chips */}
        <div className="flex gap-1 flex-wrap">
          <button
            onClick={() => setTemplateFilter('all')}
            className={`sl-chip ${templateFilter === 'all' ? 'bg-primary text-white' : 'bg-[var(--sl-panel)] text-[var(--sl-ink-secondary)]'}`}
          >
            全部 ({artifacts.length})
          </button>
          {templates.map((t) => {
            const count = artifacts.filter((a) => a.template === t).length;
            if (count === 0) return null;
            return (
              <button
                key={t}
                onClick={() => setTemplateFilter(t)}
                className={`sl-chip ${templateFilter === t ? 'bg-primary text-white' : templateColors[t] || 'bg-[var(--sl-panel)]'}`}
              >
                {getArtifactTemplateLabel(t)} ({count})
              </button>
            );
          })}
        </div>
      </div>

      {/* Artifact list */}
      <div className="divide-y divide-[var(--sl-border-subtle)]">
        {filteredArtifacts.map((artifact) => {
          const isSelected = selectedId === artifact.id;
          return (
            <button
              key={artifact.id}
              onClick={() => onSelectArtifact(artifact)}
              className={`w-full flex items-center gap-3 px-4 py-3 text-left transition-colors
                ${isSelected ? 'bg-primary/5 border-l-[3px] border-l-primary' : 'hover:bg-[var(--sl-panel)] border-l-[3px] border-l-transparent'}
              `}
            >
              <FileText size={16} className="text-[var(--sl-ink-muted)] shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium text-[var(--sl-ink)] truncate">{artifact.title}</div>
                <div className="flex items-center gap-2 mt-0.5">
                  <span className={`sl-chip text-[10px] px-1.5 h-[18px] ${templateColors[artifact.template] || ''}`}>
                    {getArtifactTemplateLabel(artifact.template)}
                  </span>
                  {artifact.subtype && (
                    <span className="text-[10px] text-[var(--sl-ink-muted)]">
                      {getArtifactSubtypeLabel(artifact.subtype)}
                    </span>
                  )}
                </div>
              </div>
              <ChevronRight size={14} className="text-[var(--sl-ink-muted)] shrink-0" />
            </button>
          );
        })}

        {filteredArtifacts.length === 0 && (
          <div className="px-4 py-8 text-center text-sm text-[var(--sl-ink-secondary)]">
            {searchTerm ? `没有匹配"${searchTerm}"的产出物` : '暂无产出物'}
          </div>
        )}
      </div>
    </div>
  );
};

export default ArtifactsView;
