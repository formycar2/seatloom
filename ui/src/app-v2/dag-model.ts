/**
 * SeatLoom DAG Workflow Data Model
 *
 * This is the canonical data structure for workflow visualization.
 * Rendering-agnostic: can be consumed by CSS layout, SVG, React Flow, or AntV X6.
 *
 * Design principles:
 * - Nodes define WHAT, edges (dependsOn) define FLOW
 * - Appearance is driven by node properties, not by rendering code
 * - Extensible via metadata bag for future needs
 */

// ─── Node Types ───

export type WorkNodeType = 'task' | 'gate' | 'milestone' | 'checkpoint' | 'decision';

export type WorkNodeStatus =
  | 'pending'      // not started, not yet reachable
  | 'waiting'      // reachable but waiting on upstream
  | 'active'       // in progress
  | 'blocked'      // in progress but stuck
  | 'done'         // completed successfully
  | 'failed'       // completed with failure
  | 'skipped'      // intentionally bypassed
  | 'revisited';   // was done, reopened

export type WorkNodePriority = 'critical' | 'high' | 'normal' | 'low';

export type WorkNodeShape = 'card' | 'diamond' | 'pill' | 'circle';

// ─── Core Node ───

export interface WorkNode {
  /** Unique node ID within the workflow */
  id: string;

  /** Node semantics */
  type: WorkNodeType;
  status: WorkNodeStatus;

  /** Display */
  label: string;
  description?: string;
  icon?: string;                    // emoji or icon key

  /** Ownership */
  owner?: string;                   // seat name
  ownerAvatar?: string;             // single char for avatar
  ownerColor?: string;              // CSS color

  /** Appearance overrides (optional — defaults derived from type/status) */
  shape?: WorkNodeShape;            // default: 'card' for task, 'diamond' for gate/decision, 'pill' for milestone
  accentColor?: string;             // override the status-derived color
  badge?: string;                   // small text badge (e.g., "P0", "3/5", "⚠")
  badgeColor?: string;

  /** DAG edges */
  dependsOn: string[];              // upstream node IDs

  /** Progress (for composite nodes) */
  progress?: {
    done: number;
    total: number;
    unit?: string;                  // e.g., "工作项", "测试", "文件"
  };

  /** Time */
  startedAt?: string;               // ISO timestamp
  completedAt?: string;
  estimatedDuration?: string;       // human-readable, e.g., "2h", "3d"
  waitingSince?: string;            // how long in current status

  /** Links */
  workItemRef?: string;             // link to WorkItem ID
  artifactRefs?: string[];          // linked artifact IDs
  url?: string;                     // external link

  /** Metadata bag for future extensions */
  meta?: Record<string, unknown>;
}

// ─── Edge (explicit, for when you need edge-level metadata) ───

export interface WorkEdge {
  from: string;
  to: string;
  type?: 'dependency' | 'feedback' | 'escalation' | 'delegation';
  label?: string;
  style?: 'solid' | 'dashed' | 'dotted';
  color?: string;
  meta?: Record<string, unknown>;
}

// ─── Workflow Container ───

export interface StageWorkflow {
  /** All nodes in this workflow */
  nodes: WorkNode[];

  /** Explicit edges (optional — if absent, edges derived from node.dependsOn) */
  edges?: WorkEdge[];

  /** Layout hints (optional — renderer may ignore) */
  layout?: {
    direction?: 'LR' | 'TB';       // left-to-right or top-to-bottom
    compact?: boolean;              // minimize whitespace
    groupBy?: 'owner' | 'type' | 'status';  // group nodes by attribute
  };
}

// ─── Computed helpers ───

/** Compute topological layers for DAG rendering */
export function computeLayers(nodes: WorkNode[]): WorkNode[][] {
  const depthMap = new Map<string, number>();

  const getDepth = (id: string): number => {
    if (depthMap.has(id)) return depthMap.get(id)!;
    const node = nodes.find(n => n.id === id);
    if (!node || node.dependsOn.length === 0) {
      depthMap.set(id, 0);
      return 0;
    }
    const d = 1 + Math.max(...node.dependsOn.map(getDepth));
    depthMap.set(id, d);
    return d;
  };

  nodes.forEach(n => getDepth(n.id));
  const maxDepth = nodes.length > 0 ? Math.max(...Array.from(depthMap.values())) : 0;

  const layers: WorkNode[][] = [];
  for (let d = 0; d <= maxDepth; d++) {
    layers.push(nodes.filter(n => depthMap.get(n.id) === d));
  }
  return layers;
}

/** Get downstream nodes (nodes that depend on the given node) */
export function getDownstream(nodeId: string, allNodes: WorkNode[]): WorkNode[] {
  return allNodes.filter(n => n.dependsOn.includes(nodeId));
}

/** Get upstream nodes */
export function getUpstream(nodeId: string, allNodes: WorkNode[]): WorkNode[] {
  const node = allNodes.find(n => n.id === nodeId);
  if (!node) return [];
  return node.dependsOn
    .map(depId => allNodes.find(n => n.id === depId))
    .filter(Boolean) as WorkNode[];
}

/** Default shape for a node type */
export function defaultShape(type: WorkNodeType): WorkNodeShape {
  switch (type) {
    case 'gate':
    case 'decision':
      return 'diamond';
    case 'milestone':
    case 'checkpoint':
      return 'pill';
    default:
      return 'card';
  }
}

/** Default color for a node status */
export function defaultStatusColor(status: WorkNodeStatus): string {
  switch (status) {
    case 'done': return 'var(--sl-green)';
    case 'active': return 'var(--sl-brand)';
    case 'blocked':
    case 'failed': return 'var(--sl-red)';
    case 'waiting':
    case 'pending': return 'var(--sl-border)';
    case 'skipped': return 'var(--sl-text-tertiary)';
    case 'revisited': return 'var(--sl-amber)';
    default: return 'var(--sl-border)';
  }
}
