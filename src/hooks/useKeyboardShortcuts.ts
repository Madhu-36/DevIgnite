import { useEffect } from 'react';

interface ShortcutOptions {
  onRefresh?: () => void;
  onOpenSettings?: () => void;
  onTriggerDoctor?: () => void;
}

export const useKeyboardShortcuts = ({
  onRefresh,
  onOpenSettings,
  onTriggerDoctor,
}: ShortcutOptions) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl+Shift+R or Ctrl+R (without default browser reload if wanted, or F5)
      if ((event.ctrlKey || event.metaKey) && event.key === 'r') {
        if (onRefresh) {
          event.preventDefault();
          onRefresh();
        }
      }

      // Ctrl+, opens settings
      if ((event.ctrlKey || event.metaKey) && event.key === ',') {
        if (onOpenSettings) {
          event.preventDefault();
          onOpenSettings();
        }
      }

      // Ctrl+Shift+D triggers Doctor
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'D') {
        if (onTriggerDoctor) {
          event.preventDefault();
          onTriggerDoctor();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onRefresh, onOpenSettings, onTriggerDoctor]);
};
