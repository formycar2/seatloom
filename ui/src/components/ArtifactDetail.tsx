import React from 'react';
import { AlertTriangle, BookOpenText, FileBadge2, Link2, ShieldCheck, Tags } from 'lucide-react';
import { Artifact } from '../types';
import { useDataStore } from '../stores/useDataStore';
import { formatDateTimeZh } from '../utils/display';
import {
  getArtifactSubtypeLabel,
  getArtifactSummaryHeading,
  getArtifactStatusLabel,
  getArtifactTemplateLabel,
  isArtifactTypingValid,
} from '../utils/artifacts';

interface ArtifactDetailProps {
  artifact: Artifact;
}

const toneByTemplate: Record<Artifact['template'], string> = {
  T1: 'bg-status-done/10 text-status-done border-status-done/20',
  T2: 'bg-status-drifted/10 text-status-drifted border-status-drifted/20',
  T3: 'bg-primary/10 text-primary border-primary/20',
  T4: 'bg-status-warning/10 text-status-warning border-status-warning/20',
  T5: 'bg-status-active/10 text-status-active border-status-active/20',
  T6: 'bg-secondary text-text-secondary border-border/40',
  T7: 'bg-status-error/10 text-status-error border-status-error/20',
};

const ArtifactDetail: React.FC<ArtifactDetailProps> = ({ artifact }) => {
  const { activeProjectId, projectData } = useDataStore();
  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const relatedWorkItem = currentData?.workItems.find((item) => item.id === artifact.source_workitem_id) || null;
  const relatedSession = currentData?.sessions.find((session) => session.id === artifact.source_session_id) || null;
  const relatedHandoff = currentData?.handoffs.find((handoff) => handoff.id === artifact.source_handoff_id) || null;
  const typingValid = isArtifactTypingValid(artifact);

  return (
    <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-300">
      <div className="space-y-4 rounded-2xl border border-border bg-card p-4 shadow-sm">
        <div className="flex items-start justify-between gap-3 flex-wrap">
          <div className="space-y-2 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <span className={`inline-flex items-center rounded-full border px-2.5 py-1 text-[10px] font-black tracking-[0.16em] uppercase ${toneByTemplate[artifact.template]}`}>
                {getArtifactTemplateLabel(artifact.template)}
              </span>
              <span className="inline-flex items-center rounded-full border border-border bg-background px-2.5 py-1 text-[10px] font-bold text-text-secondary">
                {getArtifactSubtypeLabel(artifact.subtype)}
              </span>
              {artifact.status && (
                <span className="inline-flex items-center rounded-full border border-border bg-background px-2.5 py-1 text-[10px] font-semibold text-text-muted">
                  {getArtifactStatusLabel(artifact.status)}
                </span>
              )}
            </div>
            <div>
              <div className="text-lg font-bold text-text-primary leading-tight">{artifact.title}</div>
              <div className="text-[12px] text-text-secondary leading-6 mt-1 break-all">{artifact.storage_path}</div>
            </div>
          </div>
          <div className="text-right text-[11px] text-text-muted space-y-1">
            <div>{artifact.id}</div>
            <div>索引于 {formatDateTimeZh(artifact.created_at)}</div>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-[12px]">
          <div className="rounded-xl border border-border bg-background p-3 space-y-1.5">
            <div className="meta-label">Template / subtype</div>
            <div className="font-semibold text-text-primary">
              {artifact.template} · {artifact.subtype}
            </div>
            <div className="text-text-secondary">详情渲染、筛选入口和后续自动化判断都以这组双键为准，而不是再让用户自行猜测文档类型。</div>
          </div>
          <div className="rounded-xl border border-border bg-background p-3 space-y-1.5">
            <div className="meta-label">结构化元数据</div>
            <div className="font-semibold text-text-primary">
              {artifact.author || '作者未记录'} · {artifact.date || '日期未记录'}
            </div>
            <div className="text-text-secondary">版本 {artifact.version || '未记录'}{artifact.supersedes ? ` · 替代 ${artifact.supersedes}` : ''}</div>
          </div>
        </div>
      </div>

      {!typingValid && (
        <div className="rounded-xl border border-status-warning/30 bg-status-warning/5 p-4 text-status-warning flex items-start gap-3">
          <AlertTriangle size={16} className="mt-0.5" />
          <div>
            <div className="text-sm font-bold">当前元数据无法支撑 subtype 专用渲染</div>
            <div className="text-xs leading-6 text-status-warning/90">
              该产物未通过 `DOCUMENT_TEMPLATES.md` 中的 allow-list 校验，因此在元数据修正前，界面不会启用 subtype 专属路由或 Gate 判断。
            </div>
          </div>
        </div>
      )}

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <BookOpenText size={12} /> {getArtifactSummaryHeading(artifact.template)}
        </h3>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-3">
          <p className="text-sm text-text-secondary leading-7 whitespace-pre-wrap">{artifact.summary || '该产物已经被对象化索引，但尚未补充摘要说明。'}</p>
          {artifact.summary_points && artifact.summary_points.length > 0 && (
            <ul className="space-y-2">
              {artifact.summary_points.map((point, index) => (
                <li key={`${artifact.id}-point-${index}`} className="flex items-start gap-3 text-[12px] leading-6 text-text-secondary">
                  <span className="mt-1.5 h-1.5 w-1.5 rounded-full bg-accent shrink-0" />
                  <span>{point}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <FileBadge2 size={12} /> Header 字段与路由真值
        </h3>
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-[12px] leading-6">
            <div>
              <div className="meta-label">依赖链</div>
              <div className="text-text-primary whitespace-pre-wrap">{artifact.depends_on?.join(', ') || '当前未记录显式依赖链'}</div>
            </div>
            <div>
              <div className="meta-label">标签</div>
              <div className="text-text-primary whitespace-pre-wrap">{artifact.tags?.join(', ') || '当前没有标签'}</div>
            </div>
          </div>
          {artifact.content_preview && artifact.content_preview.length > 0 && (
            <div className="space-y-2">
              <div className="meta-label">预览摘录</div>
              <div className="space-y-2">
                {artifact.content_preview.map((line, index) => (
                  <div key={`${artifact.id}-preview-${index}`} className="rounded-lg border border-border bg-background px-3 py-2 text-[12px] leading-6 text-text-secondary">
                    {line}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="space-y-3">
        <h3 className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
          <Link2 size={12} /> 关联执行上下文
        </h3>
        <div className="grid grid-cols-1 gap-3">
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5 text-[12px] leading-6">
            <div className="meta-label">工作项</div>
            <div className="text-text-primary">{relatedWorkItem ? `${relatedWorkItem.id.toUpperCase()} · ${relatedWorkItem.title}` : '当前没有把该产物挂到具体工作项上'}</div>
          </div>
          <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-1.5 text-[12px] leading-6">
            <div className="meta-label">会话 / 交接</div>
            <div className="text-text-primary">
              {relatedSession ? `${relatedSession.id.toUpperCase()} · ${relatedSession.branch || relatedSession.workspace_path}` : '当前没有记录来源会话'}
            </div>
            <div className="text-text-secondary">{relatedHandoff ? `${relatedHandoff.id.toUpperCase()} 将该产物留在交接回执链路中。` : '当前没有记录来源交接。'}</div>
          </div>
        </div>
      </div>

      <div className="rounded-xl border border-status-active/20 bg-status-active/5 p-4 text-status-active flex items-start gap-3">
        <ShieldCheck size={16} className="mt-0.5" />
        <div className="space-y-1">
          <div className="text-sm font-bold">该文档已作为可检索、可打开的 Artifact 对象接入</div>
          <div className="text-xs leading-6 text-status-active/90">
            时间线、收件箱关联证据和详情区都可以直接打开这个结构化产物，而不是只剩一条难以理解的原始路径字符串。
          </div>
        </div>
      </div>

      {artifact.tags && artifact.tags.length > 0 && (
        <div className="rounded-xl border border-border bg-bg-elevated p-4 space-y-2">
          <div className="text-[11px] font-bold text-text-muted uppercase tracking-wider flex items-center gap-1.5">
            <Tags size={12} /> 检索标签
          </div>
          <div className="flex items-center gap-2 flex-wrap">
            {artifact.tags.map((tag) => (
              <span key={`${artifact.id}-${tag}`} className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-text-secondary">
                {tag}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default ArtifactDetail;
