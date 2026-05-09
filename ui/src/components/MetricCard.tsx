import React from 'react';

interface MetricCardProps {
  label: string;
  value: number | string;
  icon: React.ElementType;
  color: 'primary' | 'success' | 'warning' | 'error' | 'review' | 'done';
  onClick?: () => void;
  onMouseEnter?: (e: React.MouseEvent) => void;
  onMouseLeave?: () => void;
}

const colorMap = {
  primary: { bg: 'bg-primary/10', text: 'text-primary', icon: 'text-primary' },
  success: { bg: 'bg-[var(--sl-success-light)]', text: 'text-[var(--sl-success)]', icon: 'text-[var(--sl-success)]' },
  warning: { bg: 'bg-[var(--sl-warning-light)]', text: 'text-[var(--sl-warning)]', icon: 'text-[var(--sl-warning)]' },
  error: { bg: 'bg-[var(--sl-error-light)]', text: 'text-[var(--sl-error)]', icon: 'text-[var(--sl-error)]' },
  review: { bg: 'bg-[var(--sl-review-light)]', text: 'text-[var(--sl-review)]', icon: 'text-[var(--sl-review)]' },
  done: { bg: 'bg-[var(--sl-done-light)]', text: 'text-[var(--sl-done)]', icon: 'text-[var(--sl-done)]' },
};

const MetricCard: React.FC<MetricCardProps> = ({ 
  label, value, icon: Icon, color, onClick, onMouseEnter, onMouseLeave 
}) => {
  const c = colorMap[color];

  return (
    <button
      onClick={onClick}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      className={`sl-card flex items-center gap-3 px-4 py-3 hover:shadow-elevated transition-all duration-150 cursor-pointer text-left w-full hover:border-primary/20`}
    >
      <div className={`w-9 h-9 rounded-lg ${c.bg} flex items-center justify-center shrink-0 shadow-sm`}>
        <Icon size={18} className={c.icon} />
      </div>
      <div className="min-w-0">
        <div className={`sl-metric ${c.text}`}>{value}</div>
        <div className="text-[10px] font-black uppercase tracking-widest text-[var(--sl-ink-muted)] truncate">{label}</div>
      </div>
    </button>
  );
};

export default MetricCard;
