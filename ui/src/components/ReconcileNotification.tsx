import React, { useEffect } from 'react';
import { RefreshCw, ArrowRight } from 'lucide-react';
import { useLocaleStore } from '../stores/useLocaleStore';

interface ReconcileNotificationProps {
  count: number;
  onClose: () => void;
  onViewInbox: () => void;
}

const ReconcileNotification: React.FC<ReconcileNotificationProps> = ({ count, onClose, onViewInbox }) => {
  const { t } = useLocaleStore();

  useEffect(() => {
    const timer = setTimeout(() => {
      onClose();
    }, 5000);
    return () => clearTimeout(timer);
  }, [onClose]);

  if (count === 0) return null;

  return (
    <div className="fixed top-16 right-4 z-[60] animate-in slide-in-from-right duration-300">
      <div className="bg-card border border-primary/20 rounded-xl shadow-2xl overflow-hidden min-w-[320px]">
        <div className="p-4 flex items-center gap-4 bg-primary/5">
          <div className="p-2 bg-primary/10 rounded-lg text-primary animate-spin-slow">
            <RefreshCw size={20} />
          </div>
          <div className="flex-1">
            <h4 className="text-sm font-bold text-foreground">
              {t.pipeline.summary.replace('{count}', count.toString())}
            </h4>
            <button 
              onClick={onViewInbox}
              className="text-xs font-semibold text-primary flex items-center gap-1 hover:underline mt-1"
            >
              {t.pipeline.viewInInbox}
              <ArrowRight size={10} />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ReconcileNotification;
