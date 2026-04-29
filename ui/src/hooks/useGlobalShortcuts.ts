import { useEffect } from 'react';

export const useGlobalShortcuts = (
  onCloseDetail: () => void,
  onToggleTab: () => void,
  onToggleTerminal: () => void,
  onShowHelp: () => void,
  onOpenCommandBar: () => void,
) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Help: ? (Shift + /) or F1
      if ((event.key === '?' && event.shiftKey) || event.key === 'F1') {
        event.preventDefault();
        onShowHelp();
        return;
      }

      if (event.key === 'Escape') {
        onCloseDetail();
      }

      // Cmd/Ctrl + K: Supervisor Command Bar
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        onOpenCommandBar();
      }

      // Cmd/Ctrl + ` : Toggle Operator Console
      if ((event.metaKey || event.ctrlKey) && event.key === '`') {
        event.preventDefault();
        onToggleTerminal();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onCloseDetail, onToggleTab, onToggleTerminal, onShowHelp, onOpenCommandBar]);
};
