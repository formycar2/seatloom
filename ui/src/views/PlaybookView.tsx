import React from 'react';
import { BookOpen, FileText, ChevronRight } from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { Artifact } from '../types';
import { getArtifactSubtypeLabel } from '../utils/artifacts';

interface PlaybookViewProps {
  onSelectArtifact: (artifact: Artifact) => void;
  selectedId?: string | null;
}

const PlaybookView: React.FC<PlaybookViewProps> = ({ onSelectArtifact, selectedId }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  const governanceDocs = (currentData?.artifacts || []).filter(
    (a) => a.template === 'T7'
  );

  if (!currentData) {
    return (
      <div className="flex-1 flex items-center justify-center text-[var(--sl-ink-secondary)]">
        <p className="text-sm">请选择一个项目</p>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="px-4 py-3 border-b border-[var(--sl-border)]">
        <h2 className="text-sm font-semibold text-[var(--sl-ink)]">Playbook & 治理文档</h2>
        <p className="text-caption text-[var(--sl-ink-secondary)] mt-0.5">
          可复用的经验和治理规则
        </p>
      </div>

      {governanceDocs.length === 0 ? (
        <div className="px-4 py-12 text-center">
          <BookOpen size={32} className="mx-auto mb-3 text-[var(--sl-ink-muted)] opacity-40" />
          <p className="text-sm text-[var(--sl-ink-secondary)]">暂无可复用的 Playbook 或治理文档</p>
        </div>
      ) : (
        <div className="divide-y divide-[var(--sl-border-subtle)]">
          {governanceDocs.map((artifact) => {
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
                  {artifact.subtype && (
                    <span className="text-xs text-[var(--sl-ink-secondary)]">
                      {getArtifactSubtypeLabel(artifact.subtype)}
                    </span>
                  )}
                </div>
                <ChevronRight size={14} className="text-[var(--sl-ink-muted)] shrink-0" />
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default PlaybookView;
