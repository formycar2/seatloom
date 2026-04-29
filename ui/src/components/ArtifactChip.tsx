import React from 'react';
import { FileText } from 'lucide-react';
import { Artifact } from '../types';
import { getArtifactSubtypeLabel, getArtifactTemplateLabel } from '../utils/artifacts';

interface ArtifactChipProps {
  artifact: Artifact;
  onClick?: () => void;
}

const ArtifactChip: React.FC<ArtifactChipProps> = ({ artifact, onClick }) => {
  const content = (
    <>
      <FileText size={11} className="text-accent shrink-0" />
      <span className="font-semibold text-text-primary truncate max-w-[180px]">{artifact.title}</span>
      <span className="text-text-muted">·</span>
      <span className="text-text-secondary whitespace-nowrap">{getArtifactTemplateLabel(artifact.template)}</span>
      <span className="text-text-muted">/</span>
      <span className="text-text-secondary whitespace-nowrap">{getArtifactSubtypeLabel(artifact.subtype)}</span>
    </>
  );

  if (onClick) {
    return (
      <button
        type="button"
        onClick={(event) => {
          event.stopPropagation();
          onClick();
        }}
        className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-border bg-background px-2.5 py-1 text-[10px] hover:border-accent/40 hover:bg-accent/5"
      >
        {content}
      </button>
    );
  }

  return <div className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-border bg-background px-2.5 py-1 text-[10px]">{content}</div>;
};

export default ArtifactChip;
