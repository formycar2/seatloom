import React, { useEffect, useRef, useState } from 'react';
import { X } from 'lucide-react';

interface BottomSheetProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
}

const BottomSheet: React.FC<BottomSheetProps> = ({ isOpen, onClose, title, children }) => {
  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/30 z-40 transition-opacity"
        onClick={onClose}
      />

      {/* Sheet */}
      <div className="fixed bottom-0 left-0 right-0 z-50 bg-canvas rounded-t-2xl shadow-overlay max-h-[85vh] flex flex-col animate-in slide-in-from-bottom duration-300">
        {/* Handle + header */}
        <div className="flex items-center justify-between px-4 pt-3 pb-2 border-b border-[var(--sl-border-subtle)]">
          <div className="flex items-center gap-3">
            <div className="w-8 h-1 rounded-full bg-[var(--sl-border)] mx-auto" />
            {title && <span className="text-sm font-semibold text-[var(--sl-ink)]">{title}</span>}
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-[var(--sl-ink-muted)] hover:bg-[var(--sl-panel)] transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {children}
        </div>
      </div>
    </>
  );
};

export default BottomSheet;
