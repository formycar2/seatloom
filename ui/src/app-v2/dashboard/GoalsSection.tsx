/**
 * GoalsSection.tsx
 * 目标和阶段区块（默认折叠）：显示项目级目标列表和当前目标下的阶段列表，提供展开/折叠按钮
 */

import React from 'react';
import { AlertTriangle, Target } from 'lucide-react';
import { ProjectChannelData, PlanPhase } from '../types';

interface GoalsSectionProps {
  goals: ProjectChannelData['goals'];
  currentGoal: ProjectChannelData['currentGoal'];
  stages: PlanPhase[];
}

export const GoalsSection: React.FC<GoalsSectionProps> = ({ goals, currentGoal, stages }) => (
  <div style={{
    marginTop: 60, paddingTop: 24, borderTop: '2px dashed var(--sl-border-light)',
    opacity: 0.6, transition: 'opacity 300ms ease',
    display: 'flex', flexDirection: 'column', gap: 16
  }}>
    <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: 'var(--sl-text-tertiary)', fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.05em' }}>
      <Target size={14} /> 宏观背景：项目目标与阶段全景
    </div>
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
      <span style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', fontWeight: 600, flexShrink: 0, textTransform: 'uppercase' }}>总目标</span>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flex: 1, overflowX: 'auto', paddingBottom: 2 }}>
        {goals.map((g, i) => (
          <React.Fragment key={g.name}>
            <span style={{
              fontSize: 12, fontWeight: g.status === 'active' ? 600 : 500,
              color: g.status === 'active' ? 'var(--sl-brand)' : g.status === 'done' ? 'var(--sl-text-secondary)' : 'var(--sl-text-tertiary)',
              padding: g.status === 'active' ? '4px 10px' : '4px 8px',
              background: g.status === 'active' ? 'var(--sl-brand-subtle)' : 'transparent',
              borderRadius: 'var(--sl-radius-full)', whiteSpace: 'nowrap',
            }}>
              {g.status === 'done' && '✓ '}{g.name}
            </span>
            {i < goals.length - 1 && <span style={{ color: 'var(--sl-border)', fontSize: 12 }}>→</span>}
          </React.Fragment>
        ))}
      </div>
    </div>
    <div style={{ padding: '14px', borderRadius: 'var(--sl-radius-md)', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)' }}>
      <div style={{ fontSize: 12, fontWeight: 600, color: 'var(--sl-text-primary)', marginBottom: 12 }}>{currentGoal.name} - 详细路线图</div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 0, flexWrap: 'wrap', rowGap: 8 }}>
        {stages.map((stage, i, arr) => {
          const dotColor = stage.status === 'done' ? 'var(--sl-green)' : stage.status === 'active' ? 'var(--sl-brand)' : stage.status === 'revisited' ? 'var(--sl-amber)' : 'var(--sl-border)';
          return (
            <React.Fragment key={stage.name}>
              <div style={{
                display: 'flex', alignItems: 'center', gap: 6, padding: '4px 10px', borderRadius: 'var(--sl-radius-full)',
                background: stage.status === 'active' ? 'var(--sl-brand-subtle)' : stage.status === 'revisited' ? 'var(--sl-amber-subtle)' : 'transparent',
                border: stage.status === 'active' ? '1px solid var(--sl-brand)' : stage.status === 'revisited' ? '1px solid var(--sl-amber)' : '1px solid transparent',
              }}>
                <div style={{ width: 8, height: 8, borderRadius: '50%', background: dotColor }} />
                <span style={{ fontSize: 12, fontWeight: stage.status === 'active' || stage.status === 'revisited' ? 600 : 500, color: stage.status === 'active' ? 'var(--sl-brand)' : stage.status === 'revisited' ? 'var(--sl-amber)' : stage.status === 'done' ? 'var(--sl-text-secondary)' : 'var(--sl-text-tertiary)', whiteSpace: 'nowrap' }}>
                  {stage.name}{stage.status === 'revisited' && ' ⟲'}
                </span>
              </div>
              {i < arr.length - 1 && <span style={{ color: 'var(--sl-border)', fontSize: 12, margin: '0 4px' }}>→</span>}
            </React.Fragment>
          );
        })}
      </div>
      {stages.filter(s => s.status === 'revisited').length > 0 && (
        <div style={{ marginTop: 12, display: 'flex', flexDirection: 'column', gap: 6 }}>
          {stages.filter(s => s.status === 'revisited').map(s => (
            <div key={s.name} style={{ padding: '8px 12px', fontSize: 11, color: 'var(--sl-amber)', background: 'var(--sl-amber-subtle)', border: '1px solid var(--sl-amber)40', borderRadius: 'var(--sl-radius-md)', display: 'flex', alignItems: 'center', gap: 6 }}>
              <AlertTriangle size={14} />
              <span style={{ fontWeight: 600 }}>{s.name} 阶段被重新打开</span>
              {s.revisitReason && <span style={{ color: 'var(--sl-text-secondary)' }}>: {s.revisitReason}</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  </div>
);
