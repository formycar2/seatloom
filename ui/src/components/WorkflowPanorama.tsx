import React from 'react';
import { useDataStore } from '../stores/useDataStore';
import DagWorkflow from '../app-v2/DagWorkflow';
import { StageWorkflow, WorkNode } from '../app-v2/dag-model';
import { WorkItem } from '../types';

interface WorkflowPanoramaProps {
  onNodeMouseEnter?: (e: React.MouseEvent, node: any) => void;
  onNodeMouseLeave?: () => void;
}

const WorkflowPanorama: React.FC<WorkflowPanoramaProps> = ({ 
  onNodeMouseEnter, 
  onNodeMouseLeave 
}) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;

  if (!currentData) return null;

  // ── Mapping Truth (WorkItems) to DAG Nodes ──
  const mapWorkItemToNode = (wi: WorkItem): WorkNode => {
    const owner = currentData.seats.find(s => s.id === wi.owner_seat_id)?.name || '未指派';
    
    // Determine status color/icon mapping
    let status: any = 'waiting';
    if (wi.status === 'Active') status = 'active';
    else if (wi.status === 'Blocked') status = 'blocked';
    else if (wi.status === 'InReview') status = 'active'; // Showing as pulse
    else if (wi.status === 'Done' || wi.status === 'Verified') status = 'done';

    return {
      id: wi.id,
      type: 'task',
      label: wi.title,
      owner: owner.charAt(0).toUpperCase() + owner.slice(1),
      ownerAvatar: owner.charAt(0).toUpperCase(),
      status: status,
      dependsOn: wi.depends_on || [],
      description: wi.goal,
      workItemRef: wi.id.toUpperCase(),
      // Adding extra fields for the hover popup
      payload: {
        summary: wi.goal,
        ac: wi.acceptance_criteria,
        updated: wi.updated_at
      }
    } as any;
  };

  const workflow: StageWorkflow = {
    nodes: currentData.workItems.map(mapWorkItemToNode),
    edges: [] // DagWorkflow will compute these from nodes.dependsOn
  };

  return (
    <div className="sl-card overflow-hidden">
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--sl-border-subtle)] bg-[var(--sl-panel)]">
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-semibold text-[var(--sl-ink)] uppercase tracking-wider">实时工作流全景 (TRUTH-DRIVEN DAG)</h3>
          <span className="text-[10px] font-black px-2 py-0.5 rounded bg-primary text-white">LIVE</span>
        </div>
        <div className="flex items-center gap-4 text-[11px] font-medium text-[var(--sl-ink-muted)]">
          <span className="flex items-center gap-1"><div className="w-2 h-2 rounded-full bg-[var(--sl-brand)]" /> 进行中</span>
          <span className="flex items-center gap-1"><div className="w-2 h-2 rounded-full bg-[var(--sl-red)]" /> 阻塞</span>
          <span className="flex items-center gap-1"><div className="w-2 h-2 rounded-full bg-[var(--sl-done)]" /> 已交付</span>
        </div>
      </div>

      <div className="p-4 bg-canvas/30" onMouseLeave={onNodeMouseLeave}>
        <DagWorkflow 
          workflow={workflow} 
          onNodeMouseEnter={onNodeMouseEnter}
        />
      </div>
    </div>
  );
};

export default WorkflowPanorama;
