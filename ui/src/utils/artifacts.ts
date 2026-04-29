import { Artifact, ArtifactSubtype, ArtifactTemplate, CanonicalEvent } from '../types';

export const ARTIFACT_SUBTYPE_ALLOW_LIST: Record<ArtifactTemplate, ArtifactSubtype[]> = {
  T1: ['prd', 'ux_spec', 'interaction_spec', 'acceptance_spec', 'architecture_design', 'architecture_decisions'],
  T2: ['seat_role'],
  T3: ['task', 'fix', 'integration', 'verification'],
  T4: ['gap_review', 'benchmark', 'process_mapping', 'design_proposal'],
  T5: ['acceptance_review', 'gate_decision'],
  T6: ['daily_log'],
  T7: ['coordination_rules', 'workflow_principles', 'collaboration_protocol', 'document_templates'],
};

const artifactTemplateLabels: Record<ArtifactTemplate, string> = {
  T1: '权威文档',
  T2: '角色说明',
  T3: '任务包',
  T4: '评审',
  T5: '验收',
  T6: '日记忆',
  T7: '治理文档',
};

const artifactSubtypeLabels: Record<ArtifactSubtype, string> = {
  prd: '产品需求',
  ux_spec: '体验规格',
  interaction_spec: '交互规格',
  acceptance_spec: '验收规格',
  architecture_design: '架构设计',
  architecture_decisions: '架构决策',
  seat_role: '席位角色',
  task: '任务',
  fix: '修复',
  integration: '集成',
  verification: '验证',
  gap_review: '缺口评审',
  benchmark: '对标评审',
  process_mapping: '过程映射',
  design_proposal: '设计提案',
  acceptance_review: '验收评审',
  gate_decision: '闸门决议',
  daily_log: '日记忆',
  coordination_rules: '协作规则',
  workflow_principles: 'AI 原生原则',
  collaboration_protocol: '协作协议',
  document_templates: '文档模板',
};

const artifactSummaryHeadings: Record<ArtifactTemplate, string> = {
  T1: '合同快照',
  T2: '角色职责快照',
  T3: '执行任务包快照',
  T4: '评审快照',
  T5: '验收快照',
  T6: '日记忆快照',
  T7: '治理规则快照',
};

const artifactStatusLabels: Record<string, string> = {
  issued: '已签发',
  delivered: '已交付',
  recovered: '已恢复',
  ready: '就绪',
  accepted: '已采纳',
  draft: '草稿',
  approved: '已批准',
  review: '评审中',
  in_progress: '进行中',
  active: '生效中',
  conditional: '条件通过',
  conditionally_adopted: '条件采纳',
  pending_review: '待评审',
};

export const getArtifactTemplateLabel = (template: ArtifactTemplate) => artifactTemplateLabels[template] ?? template;

export const getArtifactSubtypeLabel = (subtype: ArtifactSubtype) => artifactSubtypeLabels[subtype] ?? subtype;

export const getArtifactSummaryHeading = (template: ArtifactTemplate) => artifactSummaryHeadings[template] ?? '产物快照';

export const getArtifactStatusLabel = (status?: string | null) => {
  if (!status) return null;
  return artifactStatusLabels[status] ?? status;
};

export const isArtifactSubtypeAllowed = (template: ArtifactTemplate, subtype: ArtifactSubtype) => {
  return ARTIFACT_SUBTYPE_ALLOW_LIST[template]?.includes(subtype) ?? false;
};

export const isArtifactTypingValid = (artifact: Pick<Artifact, 'template' | 'subtype'>) => {
  return isArtifactSubtypeAllowed(artifact.template, artifact.subtype);
};

export const getArtifactTemplateOptions = (artifacts: Artifact[]) => {
  return Array.from(new Set(artifacts.map((artifact) => artifact.template)));
};

export const getArtifactSubtypeOptions = (artifacts: Artifact[], template?: ArtifactTemplate | null) => {
  const scoped = template ? artifacts.filter((artifact) => artifact.template === template) : artifacts;
  return Array.from(new Set(scoped.map((artifact) => artifact.subtype)));
};

export const resolveArtifactById = (artifacts: Artifact[], artifactId?: string | null) => {
  if (!artifactId) return null;
  return artifacts.find((artifact) => artifact.id === artifactId) ?? null;
};

export const resolveArtifactByPath = (artifacts: Artifact[], path?: string | null) => {
  if (!path) return null;
  return artifacts.find((artifact) => artifact.storage_path === path) ?? null;
};

export const dedupeArtifacts = (artifacts: Artifact[]) => {
  const seen = new Set<string>();
  return artifacts.filter((artifact) => {
    if (seen.has(artifact.id)) return false;
    seen.add(artifact.id);
    return true;
  });
};

export const getArtifactsFromPaths = (artifacts: Artifact[], paths: string[]) => {
  return dedupeArtifacts(paths.map((path) => resolveArtifactByPath(artifacts, path)).filter((artifact): artifact is Artifact => Boolean(artifact)));
};

export const getArtifactsFromIds = (artifacts: Artifact[], artifactIds: string[]) => {
  return dedupeArtifacts(artifactIds.map((artifactId) => resolveArtifactById(artifacts, artifactId)).filter((artifact): artifact is Artifact => Boolean(artifact)));
};

export const getUnmappedEvidencePaths = (artifacts: Artifact[], paths: string[]) => {
  return paths.filter((path) => !resolveArtifactByPath(artifacts, path));
};

export const getUnmappedArtifactIds = (artifacts: Artifact[], artifactIds: string[]) => {
  return artifactIds.filter((artifactId) => !resolveArtifactById(artifacts, artifactId));
};

export const getArtifactsForEvent = (artifacts: Artifact[], event: CanonicalEvent) => {
  const linkedById = event.object_refs
    .map((ref) => ('Artifact' in ref ? resolveArtifactById(artifacts, ref.Artifact) : null))
    .filter((artifact): artifact is Artifact => Boolean(artifact));

  const linkedByPath = getArtifactsFromPaths(artifacts, event.evidence_refs);
  return dedupeArtifacts([...linkedById, ...linkedByPath]);
};

export const getArtifactSearchText = (artifact: Artifact) => {
  return [
    artifact.id,
    artifact.title,
    artifact.template,
    artifact.subtype,
    artifact.status,
    artifact.author,
    artifact.date,
    artifact.version,
    artifact.summary,
    artifact.storage_path,
    ...(artifact.tags || []),
    ...(artifact.summary_points || []),
    ...(artifact.content_preview || []),
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase();
};
