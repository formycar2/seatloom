import React from 'react';
import { ArrowLeft } from 'lucide-react';
import { useResponsive } from '../hooks/useResponsive';

interface MasterDetailProps {
  list: React.ReactNode;
  detail: React.ReactNode | null;
  hasSelection: boolean;
  onBack: () => void;
  className?: string;
}

const MasterDetail: React.FC<MasterDetailProps> = ({
  list,
  detail,
  hasSelection,
  onBack,
  className = '',
}) => {
  const { isTablet, isMobile } = useResponsive();
  const narrowLayout = isTablet || isMobile;

  // On tablet/mobile: selection hides list entirely, detail goes full width
  if (narrowLayout && hasSelection && detail) {
    return (
      <div className={`flex flex-col flex-1 overflow-hidden ${className}`}>
        <div className="sticky top-0 z-10 bg-canvas border-b border-[var(--sl-border)] px-4 py-2.5">
          <button
            onClick={onBack}
            className="flex items-center gap-1.5 text-sm text-[var(--sl-ink-secondary)] hover:text-primary transition-colors"
          >
            <ArrowLeft size={16} />
            <span>返回列表</span>
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-4 bg-canvas">
          {detail}
        </div>
      </div>
    );
  }

  // On tablet/mobile without selection: list full width
  if (narrowLayout) {
    return (
      <div className={`flex flex-1 overflow-hidden ${className}`}>
        <div className="flex-1 overflow-y-auto bg-canvas">
          {list}
        </div>
      </div>
    );
  }

  // Desktop: side-by-side with animated list collapse
  return (
    <div className={`flex flex-1 overflow-hidden ${className}`}>
      {/* List panel */}
      <div
        className="sl-master-detail-list overflow-y-auto border-r border-[var(--sl-border)] bg-canvas shrink-0"
        style={{
          width: hasSelection ? 280 : '100%',
          minWidth: hasSelection ? 280 : 0,
        }}
      >
        {list}
      </div>

      {/* Detail panel */}
      {hasSelection && detail && (
        <div className="sl-master-detail-content flex-1 overflow-y-auto bg-canvas min-w-0">
          <div className="sticky top-0 z-10 bg-canvas/95 backdrop-blur-sm border-b border-[var(--sl-border-subtle)] px-4 py-2">
            <button
              onClick={onBack}
              className="flex items-center gap-1.5 text-xs text-[var(--sl-ink-secondary)] hover:text-primary transition-colors"
            >
              <ArrowLeft size={14} />
              <span>返回列表</span>
            </button>
          </div>
          <div className="p-4">
            {detail}
          </div>
        </div>
      )}
    </div>
  );
};

export default MasterDetail;
