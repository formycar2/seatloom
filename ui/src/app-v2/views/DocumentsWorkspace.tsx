// DocumentsWorkspace — browse the 270+ coordination markdown files ingested
// into PostgreSQL by the reconcile engine.
//
// Left rail: template (T1-T7) + subtype filters + search.
// Main: virtualised list of matches.
// Right: selected document detail with rendered markdown body and linked
// associations (workitem/handoff/session references).
//
// Markdown is rendered with `marked`. Header metadata is shown as a typed card
// above the body so T1/T2/T3/T4/T5/T6/T7 are quickly scannable.

import React, { useEffect, useMemo, useState } from 'react';
import { marked } from 'marked';
import { api, isTauri } from '../../lib/api';
import { useDataStore } from '../../stores/useDataStore';
import type { DocumentAssociationDto, DocumentDto, ReconcileResultDto } from '../../lib/types-dto';

const TEMPLATES: { key: string; label: string; tone: string }[] = [
  { key: 'T1AuthorityDoc',  label: 'T1 Authority',  tone: 'var(--sl-purple)' },
  { key: 'T2RoleProfile',   label: 'T2 Role',       tone: 'var(--sl-blue)' },
  { key: 'T3TaskPacket',    label: 'T3 Task',       tone: 'var(--sl-amber)' },
  { key: 'T4Review',        label: 'T4 Review',     tone: 'var(--sl-teal)' },
  { key: 'T5Acceptance',    label: 'T5 Acceptance', tone: 'var(--sl-green)' },
  { key: 'T6DailyMemory',   label: 'T6 Memory',     tone: 'var(--sl-text-secondary)' },
  { key: 'T7GovernanceDoc', label: 'T7 Governance', tone: 'var(--sl-red)' },
];

function toneForTemplate(template: string | null | undefined): string {
  const hit = TEMPLATES.find((t) => t.key === template);
  return hit?.tone ?? 'var(--sl-text-tertiary)';
}

export const DocumentsWorkspace: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [docs, setDocs] = useState<DocumentDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [template, setTemplate] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selected, setSelected] = useState<DocumentDto | null>(null);
  const [associations, setAssociations] = useState<DocumentAssociationDto[]>([]);
  const [reconciling, setReconciling] = useState(false);
  const [reconcileResult, setReconcileResult] = useState<ReconcileResultDto | null>(null);

  const handleReconcile = async () => {
    if (reconciling) return;
    setReconciling(true);
    setReconcileResult(null);
    try {
      const result = await api.reconcile();
      setReconcileResult(result);
      await useDataStore.getState().hydrateFromBackend();
      // Refresh local list to pick up any changes
      const rows = await api.listDocuments({ template: template ?? undefined });
      setDocs(rows);
    } catch (e) {
      setError(`reconcile: ${String(e)}`);
    } finally {
      setReconciling(false);
    }
  };

  // Initial load (or when template changes).
  useEffect(() => {
    if (!isTauri()) {
      setError('Tauri backend not available — run `pnpm tauri dev`.');
      setLoading(false);
      return;
    }
    setLoading(true);
    setError(null);
    api.listDocuments({ template: template ?? undefined })
      .then((rows) => setDocs(rows))
      .catch((e) => setError(`list documents: ${String(e)}`))
      .finally(() => setLoading(false));
  }, [template]);

  // Filtered view by search. Search matches title, file_path, tags, body (if loaded).
  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return docs;
    return docs.filter((d) => {
      if (d.title.toLowerCase().includes(q)) return true;
      if (d.filePath.toLowerCase().includes(q)) return true;
      if (d.tags.some((t) => t.toLowerCase().includes(q))) return true;
      if (d.author?.toLowerCase().includes(q)) return true;
      if (d.subtype?.toLowerCase().includes(q)) return true;
      return false;
    });
  }, [docs, search]);

  // Load full detail (including bodyText if missing) on selection.
  useEffect(() => {
    if (!selectedId) { setSelected(null); setAssociations([]); return; }
    if (!isTauri()) return;
    let cancelled = false;
    Promise.all([
      api.getDocument(selectedId),
      api.listDocumentAssociations(selectedId),
    ]).then(([doc, assocs]) => {
      if (cancelled) return;
      setSelected(doc);
      setAssociations(assocs);
    }).catch((e) => setError(`get document: ${String(e)}`));
    return () => { cancelled = true; };
  }, [selectedId]);

  return (
    <div style={{
      display: 'flex', flex: 1, overflow: 'hidden', background: 'var(--sl-bg)',
    }}>
      {/* ═══ Filter rail ═══ */}
      <aside style={{
        width: 240, borderRight: '1px solid var(--sl-border)', padding: 16,
        display: 'flex', flexDirection: 'column', gap: 16,
        background: 'var(--sl-surface)', overflow: 'auto',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--sl-text-primary)' }}>
            Coordination Docs
          </div>
          <button
            onClick={onClose}
            title="Close"
            style={{
              padding: '2px 8px', fontSize: 12, background: 'transparent',
              border: '1px solid var(--sl-border)', borderRadius: 4,
              color: 'var(--sl-text-secondary)', cursor: 'pointer',
            }}
          >×</button>
        </div>

        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Filter by title, path, tag…"
          style={{
            padding: '6px 10px', fontSize: 13, borderRadius: 4,
            border: '1px solid var(--sl-border)', background: 'var(--sl-bg)',
            color: 'var(--sl-text-primary)', outline: 'none',
          }}
        />

        <button
          onClick={handleReconcile}
          disabled={reconciling}
          style={{
            padding: '6px 10px', fontSize: 13, borderRadius: 4,
            border: '1px solid var(--sl-border)', background: 'var(--sl-bg)',
            color: 'var(--sl-text-primary)', cursor: reconciling ? 'not-allowed' : 'pointer',
            opacity: reconciling ? 0.6 : 1,
            display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 6,
          }}
        >
          {reconciling ? '正在同步…' : '🔄 同步文档'}
        </button>

        {reconcileResult && (
          <div style={{
            fontSize: 11, padding: '8px 10px', borderRadius: 4,
            border: `1px solid ${reconcileResult.failed > 0 || reconcileResult.conflicted > 0 ? 'var(--sl-red)' : 'var(--sl-border-light)'}`,
            color: reconcileResult.failed > 0 || reconcileResult.conflicted > 0 ? 'var(--sl-red)' : 'var(--sl-text-secondary)',
            background: reconcileResult.failed > 0 || reconcileResult.conflicted > 0 ? 'rgba(255,123,114,0.05)' : 'transparent',
            lineHeight: 1.5,
          }}>
            已扫描 {reconcileResult.scanned} · 新增 {reconcileResult.inserted} · 更新 {reconcileResult.updated} · 未变 {reconcileResult.unchanged} · 失败 {reconcileResult.failed} · 冲突 {reconcileResult.conflicted}
          </div>
        )}

        <div>
          <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)', marginBottom: 8 }}>
            Template
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <button
              onClick={() => setTemplate(null)}
              style={{
                textAlign: 'left', padding: '6px 10px', fontSize: 13,
                background: template === null ? 'var(--sl-surface-hover)' : 'var(--sl-bg)',
                border: '1px solid var(--sl-border-light)', borderRadius: 4,
                color: 'var(--sl-text-primary)', cursor: 'pointer',
              }}
            >All templates</button>
            {TEMPLATES.map((t) => (
              <button
                key={t.key}
                onClick={() => setTemplate(t.key)}
                style={{
                  textAlign: 'left', padding: '6px 10px', fontSize: 13,
                  background: template === t.key ? 'var(--sl-surface-hover)' : 'var(--sl-bg)',
                  border: '1px solid var(--sl-border-light)', borderRadius: 4,
                  color: 'var(--sl-text-primary)', cursor: 'pointer',
                  display: 'flex', alignItems: 'center', gap: 8,
                }}
              >
                <span style={{ width: 8, height: 8, borderRadius: '50%', background: t.tone }} />
                {t.label}
              </button>
            ))}
          </div>
        </div>

        <div style={{ marginTop: 'auto', fontSize: 11, color: 'var(--sl-text-tertiary)' }}>
          {loading ? 'Loading…' : `${filtered.length} / ${docs.length} docs`}
        </div>
        {error && (
          <div style={{ padding: 8, fontSize: 12, color: 'var(--sl-red)', background: 'rgba(255,123,114,0.08)', border: '1px solid var(--sl-red)', borderRadius: 4 }}>
            {error}
          </div>
        )}
      </aside>

      {/* ═══ List ═══ */}
      <div style={{
        width: 340, borderRight: '1px solid var(--sl-border)',
        overflow: 'auto', display: 'flex', flexDirection: 'column',
      }}>
        {filtered.length === 0 && !loading && (
          <div style={{ padding: 24, fontSize: 13, color: 'var(--sl-text-tertiary)' }}>
            No matching documents.
          </div>
        )}
        {filtered.map((d) => (
          <div
            key={d.id}
            onClick={() => setSelectedId(d.id)}
            style={{
              padding: '10px 14px', borderBottom: '1px solid var(--sl-border-light)',
              cursor: 'pointer',
              background: d.id === selectedId ? 'var(--sl-surface-hover)' : 'transparent',
              display: 'flex', flexDirection: 'column', gap: 4,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{
                width: 6, height: 6, borderRadius: '50%', background: toneForTemplate(d.template),
                flexShrink: 0,
              }} />
              <span style={{ fontSize: 13, fontWeight: 600, color: 'var(--sl-text-primary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {d.title}
              </span>
            </div>
            <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', display: 'flex', gap: 8 }}>
              <span>{d.template ?? '—'}</span>
              {d.subtype && <span>· {d.subtype}</span>}
              {d.author && <span>· {d.author}</span>}
              {d.docDate && <span>· {d.docDate}</span>}
            </div>
            <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {d.filePath}
            </div>
          </div>
        ))}
      </div>

      {/* ═══ Detail ═══ */}
      <main style={{ flex: 1, overflow: 'auto', padding: 24 }}>
        {!selected && (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--sl-text-tertiary)', gap: 10, textAlign: 'center' }}>
            <div style={{ fontSize: 40, opacity: 0.15 }}>📄</div>
            <div style={{ fontSize: 14, color: 'var(--sl-text-secondary)' }}>
              Select a document to view its full body and associations.
            </div>
          </div>
        )}
        {selected && (
          <DocumentDetail doc={selected} associations={associations} />
        )}
      </main>
    </div>
  );
};

const DocumentDetail: React.FC<{
  doc: DocumentDto;
  associations: DocumentAssociationDto[];
}> = ({ doc, associations }) => {
  const body = doc.bodyText ?? '';
  const rendered = useMemo(() => {
    // marked returns HTML; cast to string (async mode disabled).
    return marked.parse(body, { async: false }) as string;
  }, [body]);

  return (
    <article style={{ maxWidth: 820, margin: '0 auto', display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Metadata card */}
      <header style={{
        padding: 16, border: '1px solid var(--sl-border)', borderRadius: 8,
        background: 'var(--sl-surface)', display: 'flex', flexDirection: 'column', gap: 8,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ width: 8, height: 8, borderRadius: '50%', background: toneForTemplate(doc.template) }} />
          <span style={{ fontSize: 11, fontWeight: 600, color: 'var(--sl-text-tertiary)' }}>
            {doc.template ?? 'NO TEMPLATE'}{doc.subtype ? ` · ${doc.subtype}` : ''}
          </span>
          <span style={{ marginLeft: 'auto', fontSize: 11, color: 'var(--sl-text-tertiary)' }}>rev {doc.revision}</span>
        </div>
        <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--sl-text-primary)', margin: 0 }}>{doc.title}</h1>
        <div style={{ fontSize: 12, color: 'var(--sl-text-secondary)', display: 'flex', gap: 14, flexWrap: 'wrap' }}>
          {doc.author && <span><strong>Author:</strong> {doc.author}</span>}
          {doc.status && <span><strong>Status:</strong> {doc.status}</span>}
          {doc.docDate && <span><strong>Date:</strong> {doc.docDate}</span>}
          {doc.version && <span><strong>Version:</strong> {doc.version}</span>}
        </div>
        <div style={{ fontSize: 11, color: 'var(--sl-text-tertiary)', fontFamily: 'ui-monospace, monospace' }}>
          {doc.filePath}
        </div>
        {doc.tags.length > 0 && (
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
            {doc.tags.map((t) => (
              <span key={t} style={{ fontSize: 11, padding: '2px 8px', background: 'var(--sl-surface-hover)', borderRadius: 4, color: 'var(--sl-text-secondary)' }}>
                {t}
              </span>
            ))}
          </div>
        )}
      </header>

      {/* Associations */}
      {associations.length > 0 && (
        <section style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <div style={{ fontSize: 11, fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.04em', color: 'var(--sl-text-tertiary)' }}>
            Linked objects
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {associations.map((a) => (
              <span key={a.id} style={{ fontSize: 12, padding: '3px 10px', background: 'var(--sl-surface)', border: '1px solid var(--sl-border-light)', borderRadius: 4 }}>
                {a.assocType} · {a.assocId}
              </span>
            ))}
          </div>
        </section>
      )}

      {/* Rendered body */}
      <section
        className="sl-doc-body"
        style={{ fontSize: 14, lineHeight: 1.65, color: 'var(--sl-text-primary)' }}
        dangerouslySetInnerHTML={{ __html: rendered }}
      />

      <style>{`
        .sl-doc-body h1, .sl-doc-body h2, .sl-doc-body h3, .sl-doc-body h4 {
          color: var(--sl-text-primary); font-weight: 700; margin-top: 1.4em; margin-bottom: 0.5em;
        }
        .sl-doc-body h1 { font-size: 22px; border-bottom: 1px solid var(--sl-border); padding-bottom: 6px; }
        .sl-doc-body h2 { font-size: 18px; }
        .sl-doc-body h3 { font-size: 16px; }
        .sl-doc-body h4 { font-size: 14px; }
        .sl-doc-body p { margin: 0.7em 0; }
        .sl-doc-body code {
          padding: 1px 5px; background: var(--sl-surface-hover); border-radius: 3px;
          font-family: ui-monospace, 'SF Mono', monospace; font-size: 13px;
        }
        .sl-doc-body pre {
          padding: 12px; background: var(--sl-surface); border-radius: 6px; overflow: auto;
          border: 1px solid var(--sl-border-light);
        }
        .sl-doc-body pre code { padding: 0; background: transparent; }
        .sl-doc-body table { border-collapse: collapse; width: 100%; margin: 0.8em 0; font-size: 13px; }
        .sl-doc-body th, .sl-doc-body td {
          padding: 6px 10px; border: 1px solid var(--sl-border-light); text-align: left;
        }
        .sl-doc-body th { background: var(--sl-surface); font-weight: 600; }
        .sl-doc-body ul, .sl-doc-body ol { padding-left: 1.4em; }
        .sl-doc-body li { margin: 0.2em 0; }
        .sl-doc-body blockquote {
          margin: 0.8em 0; padding: 0.4em 1em;
          border-left: 3px solid var(--sl-border); color: var(--sl-text-secondary);
        }
        .sl-doc-body a { color: var(--sl-brand); text-decoration: none; }
        .sl-doc-body a:hover { text-decoration: underline; }
        .sl-doc-body hr { border: none; border-top: 1px solid var(--sl-border); margin: 1.6em 0; }
      `}</style>
    </article>
  );
};
