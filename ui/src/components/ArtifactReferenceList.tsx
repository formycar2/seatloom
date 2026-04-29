import React from 'react';
import { AlertTriangle, ArrowUpRight, FileText } from 'lucide-react';
import { Artifact } from '../types';
import { formatShortDateTimeZh } from '../utils/display';
import { getArtifactStatusLabel, getArtifactSubtypeLabel, getArtifactTemplateLabel } from '../utils/artifacts';

interface ArtifactReferenceListProps {
  artifacts: Artifact[];
  onOpenArtifact: (artifactId: string) => void;
  emptyText: string;
  unmappedRefs?: string[];
}

const ArtifactReferenceList: React.FC<ArtifactReferenceListProps> = ({ artifacts, onOpenArtifact, emptyText, unmappedRefs = [] }) => {
  if (artifacts.length === 0 && unmappedRefs.length === 0) {
    return <div className="rounded-xl border border-dashed border-border p-4 text-sm text-text-secondary">{emptyText}</div>;
  }

  return (
    <div className="space-y-2.5">
      {artifacts.map((artifact) => (
        <button
          key={artifact.id}
          type="button"
          onClick={() => onOpenArtifact(artifact.id)}
          className="w-full rounded-xl border border-border bg-bg-elevated p-3 text-left transition-all hover:border-accent/30 hover:bg-accent/5"
        >
          <div className="flex items-start gap-3">
            <div className="mt-0.5 rounded-lg bg-accent/10 p-2 text-accent">
              <FileText size={14} />
            </div>
            <div className="min-w-0 flex-1 space-y-2">
              <div className="flex items-center gap-2 flex-wrap">
                <span className="rounded-full border border-border bg-background px-2 py-0.5 text-[10px] font-black tracking-[0.12em] uppercase text-text-secondary">
                  {getArtifactTemplateLabel(artifact.template)}
                </span>
                <span className="rounded-full border border-border bg-background px-2 py-0.5 text-[10px] font-semibold text-text-secondary">
                  {getArtifactSubtypeLabel(artifact.subtype)}
                </span>
                {artifact.status && (
                  <span className="rounded-full border border-border bg-background px-2 py-0.5 text-[10px] text-text-muted">
                    {getArtifactStatusLabel(artifact.status)}
                  </span>
                )}
                <span className="ml-auto inline-flex items-center gap-1 text-[10px] font-semibold text-accent">
                  打开详情 <ArrowUpRight size={11} />
                </span>
              </div>
              <div className="text-sm font-semibold text-text-primary leading-6">{artifact.title}</div>
              <div className="text-[11px] leading-5 text-text-secondary break-all">{artifact.storage_path}</div>
              <div className="flex items-center gap-2 flex-wrap text-[10px] text-text-muted">
                {artifact.author && <span>{artifact.author}</span>}
                {artifact.date && <span>{artifact.date}</span>}
                <span>索引于 {formatShortDateTimeZh(artifact.created_at)}</span>
              </div>
            </div>
          </div>
        </button>
      ))}

      {unmappedRefs.map((ref) => (
        <div key={ref} className="rounded-xl border border-status-warning/30 bg-status-warning/5 p-3 text-status-warning">
          <div className="flex items-start gap-3">
            <AlertTriangle size={14} className="mt-0.5 shrink-0" />
            <div className="space-y-1">
              <div className="text-[11px] font-semibold">尚未建立对象化映射的引用</div>
              <div className="text-[11px] leading-5 break-all text-status-warning/90">{ref}</div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

export default ArtifactReferenceList;
