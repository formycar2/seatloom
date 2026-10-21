import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  AlertCircle,
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronRight,
  CopyPlus,
  Inbox,
  Link2,
  Pause,
  Play,
  Plus,
  RefreshCw,
  Search,
  Send,
} from 'lucide-react';
import { api } from '../lib/api';
import { useDataStore } from '../stores/useDataStore';
import { CanonicalEventDto, DispatchDto, DispatchDependencyDto, SeatDto } from '../lib/types-dto';
import { eventToText } from '../app-v2/lib/eventText';
import AcceptanceSection from '../components/AcceptanceSection';
import OperatorDecisionSection from '../components/OperatorDecisionSection';

// DispatchInboxView — the governed-dispatch (G19) seat inbox. Reads the live
// `dispatches` projection via cmd_list_dispatches. Mirrors the V1 AppShell
// visual conventions (same tokens as TimelineView).
//
// ── Why this shape ──────────────────────────────────────────────────────────
// The previous layout stacked one tall card per dispatch under a seat heading.
// Operator feedback: hard to scan, too little information per screen, and the
// interactions (filter only) did not match the questions actually asked of a
// queue — "what is each seat doing, what is stuck, what is next".
//
// A dispatch queue is a *table*, not a feed. Every fact about a dispatch is
// short, comparable and belongs in a column: seat, status, priority, elapsed,
// issued-at, dependency count. So the redesign is a dense one-line-per-dispatch
// table with sortable columns, a per-seat status strip as the group header
// (triage at a glance), done rows folded away by default, and a row that
// expands in place to the full payload + provenance instead of a detail page.
//
// Below the inbox sits the D2 「待验收」 section (AcceptanceSection): the same
// page, the same 5s cadence, the other half of the loop — work that is already
// delivered and Flux-verified, waiting on the user's 接受/打回. The inbox
// answers "what should a seat do next"; 待验收 answers "do I take this work".
//
// A status kanban was rejected: dispatch titles and payload notes are long
// text, so four narrow columns would truncate the very content the operator
// reads, and cross-seat priority comparison — the thing that matters when the
// fleet is saturated — is impossible without dragging cards around.

const STATUS_STYLE: Record<string, { label: string; cls: string; dot: string }> = {
  in_flight: {
    label: '进行中',
    cls: 'bg-blue-500/15 text-blue-600 border-blue-500/30',
    dot: 'bg-blue-500',
  },
  ready: {
    label: '就绪',
    cls: 'bg-emerald-500/15 text-emerald-600 border-emerald-500/30',
    dot: 'bg-emerald-500',
  },
  blocked: {
    label: '阻塞',
    cls: 'bg-amber-500/15 text-amber-600 border-amber-500/30',
    dot: 'bg-amber-500',
  },
  done: {
    label: '完成',
    cls: 'bg-secondary text-text-secondary border-border/40',
    dot: 'bg-border',
  },
};

const STATUS_FILTERS: { key: string; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'in_flight', label: '进行中' },
  { key: 'ready', label: '就绪' },
  { key: 'blocked', label: '阻塞' },
  { key: 'done', label: '完成' },
];

// Lifecycle order — the order a dispatch actually moves through, and the
// order the operator scans in. Also the view's default secondary sort.
const STATUS_RANK: Record<string, number> = { in_flight: 0, ready: 1, blocked: 2, done: 3 };

type SortKey = 'seat' | 'status' | 'priority' | 'elapsed' | 'issued';
type SortDir = 'asc' | 'desc';

interface SortSpec {
  key: SortKey;
  hint: string;
  defaultDir: SortDir;
}

interface ColDef {
  key: string;
  label: string;
  /** Tailwind width class; empty = the flexible column that absorbs the rest. */
  width: string;
  align?: 'right';
  sort?: SortSpec;
}

// One declaration drives the <colgroup>, the header cells and the sort
// behaviour, so a column can never drift out of alignment with its control.
const COLUMNS: ColDef[] = [
  {
    key: 'seat',
    label: '席位',
    width: 'w-[132px]',
    sort: { key: 'seat', hint: '按目标席位排序', defaultDir: 'asc' },
  },
  {
    key: 'status',
    label: '状态',
    width: 'w-[84px]',
    sort: { key: 'status', hint: '按生命周期排序:进行中 → 就绪 → 阻塞 → 完成', defaultDir: 'asc' },
  },
  {
    key: 'priority',
    label: '优先级',
    width: 'w-[64px]',
    align: 'right',
    sort: { key: 'priority', hint: '数值越大越先被拉取(默认 0)', defaultDir: 'desc' },
  },
  { key: 'title', label: '派单', width: '' },
  {
    key: 'elapsed',
    label: '时长',
    width: 'w-[100px]',
    align: 'right',
    sort: {
      key: 'elapsed',
      hint: '在该状态下已停留多久(进行中/等待)或实际耗时(完成)',
      defaultDir: 'desc',
    },
  },
  {
    key: 'issued',
    label: '发出',
    width: 'w-[88px]',
    align: 'right',
    sort: { key: 'issued', hint: '派单创建时间', defaultDir: 'desc' },
  },
  { key: 'deps', label: '依赖', width: 'w-[72px]' },
  { key: 'actions', label: '', width: 'w-[60px]', align: 'right' },
];

const COL_WIDTHS = COLUMNS.map((c) => c.width);
const SORT_SPECS = COLUMNS.filter((c) => c.sort !== undefined).map((c) => c.sort!);
const LABEL_BY_SORT = new Map<SortKey, string>(
  COLUMNS.filter((c) => c.sort).map((c) => [c.sort!.key, c.label]),
);

// Deterministic per-seat tint, so a seat keeps its colour across refreshes and
// across views — the operator learns "the teal one is Nimbus".
const AVATAR_TINTS = [
  'bg-blue-500/15 text-blue-600',
  'bg-emerald-500/15 text-emerald-600',
  'bg-amber-500/15 text-amber-600',
  'bg-violet-500/15 text-violet-600',
  'bg-rose-500/15 text-rose-600',
  'bg-teal-500/15 text-teal-600',
];

function statusRank(status: string): number {
  return STATUS_RANK[status] ?? 9;
}

function statusStyle(status: string): { label: string; cls: string; dot: string } {
  return (
    STATUS_STYLE[status] ?? {
      label: status,
      cls: 'bg-secondary text-text-secondary border-border/40',
      dot: 'bg-border',
    }
  );
}

function payloadNote(payload: unknown): string {
  if (!payload || typeof payload !== 'object') return '';
  const p = payload as Record<string, unknown>;
  const raw = p.note ?? p.instructions ?? p.packet ?? p.description ?? p.summary;
  return typeof raw === 'string' ? raw : '';
}

function fmtTime(iso: string | null): string {
  if (!iso) return '—';
  const d = new Date(iso);
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${d.getMonth() + 1}/${d.getDate()} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

function fmtClock(ts: number): string {
  const d = new Date(ts);
  const p = (n: number) => n.toString().padStart(2, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

function fmtElapsed(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  if (s < 60) return `${s}秒`;
  const m = Math.floor(s / 60);
  const rs = s % 60;
  if (m < 60) return rs ? `${m}分${rs}秒` : `${m}分`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  if (h < 24) return rm ? `${h}时${rm}分` : `${h}时`;
  const d = Math.floor(h / 24);
  return `${d}天${h % 24}时`;
}

// One number per dispatch that answers "how long has this been where it is":
// done → actual run time, everything else → time in the current state.
function elapsedMs(d: DispatchDto, now: number): number {
  const end = d.completedAt ? new Date(d.completedAt).getTime() : now;
  const start = new Date(d.startedAt ?? d.issuedAt).getTime();
  return Math.max(0, end - start);
}

function elapsedNoun(d: DispatchDto): string {
  if (d.status === 'done') return '耗时';
  if (d.status === 'in_flight') return '已进行';
  return '已等待';
}

function tintFor(id: string): string {
  let h = 0;
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) % 9973;
  return AVATAR_TINTS[h % AVATAR_TINTS.length];
}

function monogram(label: string, id: string): string {
  const m = (label || id).match(/[A-Za-z0-9\u4e00-\u9fa5]/);
  return (m ? m[0] : '?').toUpperCase();
}

/**
 * How many of the newest ledger events a dependency lookup covers.
 *
 * `cmd_list_events` is the only read the UI has for this, and it is BOUNDED — the
 * repository function is `... ORDER BY occurred_at DESC LIMIT $1`. So a referent
 * found in the window is a fact; a referent NOT found in it is NOT a fact, and the
 * UI must not say "does not exist". The bound is stated wherever the answer is
 * shown, so nobody reads absence as proof.
 */
const EVENT_WINDOW = 500;

/** One dependency, resolved into something a person can read. */
interface ResolvedDep {
  dep: DispatchDependencyDto;
  /** The event, when the newest EVENT_WINDOW rows contain it. */
  event?: CanonicalEventDto;
}

/**
 * Resolve `event:<id>` dependencies against the newest EVENT_WINDOW ledger rows.
 *
 * What it can say: what that event is, who did it, when, and — through the shared
 * translator — one line of plain language. What it cannot say from here: whether a
 * referent outside the window exists. The unbounded existence answer lives in the
 * MCP `get_dispatch` tool, which is not registered as a Tauri command in this tree
 * (measured: no `cmd_get_dispatch` in src-tauri), so this view does not pretend to
 * it. A missing referent renders as "not in the newest N" — never as "not
 * satisfied" and never as "satisfied".
 */
function useResolvedDeps(deps: DispatchDependencyDto[]): ResolvedDep[] {
  const [events, setEvents] = useState<CanonicalEventDto[]>([]);

  // Only event-kind deps need the ledger; a dispatch-kind dep is resolved from the
  // dispatch list the view already holds.
  const eventValues = useMemo(
    () =>
      deps
        .filter((d) => d.depKind === 'event')
        .map((d) => d.depValue)
        .join('\u0000'),
    [deps],
  );

  // Scoped to the selected project. This window exists to render the human text of
  // the events this inbox's dependency edges name, so reading every project's
  // events would resolve edges against rows the operator is not looking at.
  const activeProjectId = useDataStore((s) => s.activeProjectId);

  useEffect(() => {
    let alive = true;
    if (!activeProjectId) {
      setEvents([]);
      return;
    }
    api
      .listEvents(activeProjectId, EVENT_WINDOW)
      .then((rows) => {
        if (alive) setEvents(rows);
      })
      .catch(() => {
        if (alive) setEvents([]);
      });
    return () => {
      alive = false;
    };
  }, [eventValues, activeProjectId]);

  return useMemo(() => {
    const byId = new Map(events.map((e) => [e.id, e]));
    return deps.map((dep) => ({
      dep,
      event: dep.depKind === 'event' ? byId.get(dep.depValue) : undefined,
    }));
  }, [deps, events]);
}

/** One dependency as a human sentence. The id may appear, but never alone. */
function depSentence(r: ResolvedDep): string {
  const { dep, event } = r;
  if (dep.depKind !== 'event') {
    // A dispatch-kind dep: the row's own seat/status columns already say what it
    // is, so this names the kind and points at the row rather than repeating it.
    return `等另一张派单(${dep.depValue})`;
  }
  if (!event) {
    // NOT a claim of non-existence. `api.listEvents` is bounded
    // (`ORDER BY occurred_at DESC LIMIT $1`), so absence from the window is absence
    // of evidence, and rendering it as evidence is the same lie shape this project
    // keeps paying for. The unbounded answer lives in the DTO's `referent` field,
    // which is not in this tree (measured: it is on
    // packet/nimbus/gate-vs-announcement @ 942da99, not on track and not here), so
    // this says what it can and names what it cannot.
    return (
      `还没读出来它在等什么:这条不在最近 ${EVENT_WINDOW} 条账本记录里。` +
      `注意 —— 这不是说它不存在,只是这个界面读不到那么远;别把「没查到」当成「没有」。`
    );
  }
  // Found: what it was, who did it, when — one plain sentence, from the same
  // translator the supervisor feed uses so the two surfaces cannot drift.
  return `已经在账本里了:${eventToText(event)}`;
}

// ── Presentational pieces ───────────────────────────────────────────────────

const SeatAvatar: React.FC<{ label: string; id: string; size?: 'sm' | 'md' }> = ({
  label,
  id,
  size = 'sm',
}) => (
  <span
    className={`shrink-0 inline-flex items-center justify-center rounded-md font-black ${tintFor(id)} ${
      size === 'sm' ? 'w-[18px] h-[18px] text-[9px]' : 'w-6 h-6 text-[10px]'
    }`}
    title={id}
  >
    {monogram(label, id)}
  </span>
);

const StatusPill: React.FC<{ status: string }> = ({ status }) => {
  const st = statusStyle(status);
  return (
    <span
      className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[10px] font-black border whitespace-nowrap ${st.cls}`}
    >
      <span className={`w-1 h-1 rounded-full ${st.dot}`} />
      {st.label}
    </span>
  );
};

interface SortHeaderCellProps {
  col: ColDef;
  active: boolean;
  dir: SortDir;
  onSort: (key: SortKey) => void;
}

const SortHeaderCell: React.FC<SortHeaderCellProps> = ({ col, active, dir, onSort }) => (
  <th className={`px-2 py-1.5 ${col.width} ${col.align === 'right' ? 'text-right' : 'text-left'}`}>
    <button
      type="button"
      onClick={() => onSort(col.sort!.key)}
      title={col.sort!.hint}
      aria-label={`按${col.label}排序`}
      className={`group inline-flex items-center gap-1 text-[10px] font-black uppercase tracking-widest transition-colors ${
        col.align === 'right' ? 'flex-row-reverse' : ''
      } ${active ? 'text-primary' : 'text-text-secondary hover:text-text-primary'}`}
    >
      {col.label}
      {active ? (
        dir === 'asc' ? (
          <ArrowUp size={10} />
        ) : (
          <ArrowDown size={10} />
        )
      ) : (
        <ArrowUpDown size={10} className="opacity-0 group-hover:opacity-40" />
      )}
    </button>
  </th>
);

// The header row, generated from the same COLUMNS declaration as the colgroup,
// so a column and its control can never drift apart.
const TableHead: React.FC<{ sortKey: SortKey; sortDir: SortDir; onSort: (k: SortKey) => void }> = ({
  sortKey,
  sortDir,
  onSort,
}) => (
  <thead>
    <tr className="border-b border-border/30">
      {COLUMNS.map((c) =>
        c.sort ? (
          <SortHeaderCell
            key={c.key}
            col={c}
            active={sortKey === c.sort.key}
            dir={sortDir}
            onSort={onSort}
          />
        ) : (
          <th
            key={c.key}
            className={`px-2 py-1.5 ${c.width} ${c.align === 'right' ? 'text-right' : 'text-left'}`}
          >
            {c.label ? (
              <span className="text-[10px] font-black text-text-secondary uppercase tracking-widest">
                {c.label}
              </span>
            ) : (
              <span className="sr-only">操作</span>
            )}
          </th>
        ),
      )}
    </tr>
  </thead>
);

interface DispatchRowProps {
  d: DispatchDto;
  seatLabel: string;
  now: number;
  expanded: boolean;
  flash: boolean;
  onToggle: (id: string) => void;
  onReuse: (d: DispatchDto) => void;
}

const DispatchRow: React.FC<DispatchRowProps> = ({
  d,
  seatLabel,
  now,
  expanded,
  flash,
  onToggle,
  onReuse,
}) => {
  const note = payloadNote(d.payload);
  const elapsed = elapsedMs(d, now);
  const highPriority = d.priority >= 5;
  // Resolved here, not in a helper: a hook has to live in a component. One ledger
  // read per mounted row, keyed on this row's event-kind dep values.
  const resolved = useResolvedDeps(d.deps);
  const depsSentence = resolved.map(depSentence).join('\n');

  return (
    <>
      <tr
        onClick={() => onToggle(d.id)}
        title={`编号 ${d.id}`}
        className={`group cursor-pointer border-b border-border/20 align-middle transition-colors duration-500 ${
          flash ? 'bg-primary/10' : expanded ? 'bg-accent/20' : 'hover:bg-secondary/40'
        }`}
      >
        {/* Seat */}
        <td className="px-2 py-1.5">
          <div className="flex items-center gap-1.5 min-w-0">
            <SeatAvatar label={seatLabel} id={d.targetSeatId} />
            <span className="text-[12px] font-bold text-text-primary truncate">{seatLabel}</span>
          </div>
        </td>
        {/* Status */}
        <td className="px-2 py-1.5">
          <StatusPill status={d.status} />
        </td>
        {/* Priority */}
        <td className="px-2 py-1.5 text-right">
          <span
            className={`font-mono text-[12px] tabular-nums ${highPriority ? 'font-black text-primary' : 'text-text-secondary'}`}
            title={`优先级 ${d.priority} — 数值越大越先被拉取`}
          >
            P{d.priority}
          </span>
        </td>
        {/* Title + payload preview (the flexible column). The cell is a real
            button so the expansion is keyboard-reachable without turning every
            row into a button in the a11y tree. */}
        <td className="px-2 py-1.5 min-w-0">
          <button
            type="button"
            onClick={(e) => {
              // The row itself is also a click target; without this the event
              // would bubble and toggle the row twice (open, then close).
              e.stopPropagation();
              onToggle(d.id);
            }}
            aria-expanded={expanded}
            className="block w-full min-w-0 text-left"
          >
            <span className="block text-[13px] font-bold text-text-primary truncate leading-5">
              {d.title}
            </span>
            {note && (
              <span
                className="block text-[11px] text-text-secondary truncate leading-4"
                title={note}
              >
                {note}
              </span>
            )}
          </button>
        </td>
        {/* Elapsed */}
        <td className="px-2 py-1.5 text-right">
          <span
            className="block font-mono text-[12px] tabular-nums text-text-primary"
            title={`${elapsedNoun(d)} ${fmtElapsed(elapsed)}`}
          >
            {fmtElapsed(elapsed)}
          </span>
        </td>
        {/* Issued at */}
        <td className="px-2 py-1.5 text-right">
          <span
            className="font-mono text-[11px] tabular-nums text-text-secondary"
            title={d.issuedAt}
          >
            {fmtTime(d.issuedAt)}
          </span>
        </td>
        {/* Dependencies — direction + count, never drawn edges. The count badge is
            unchanged (he scans by it); the tooltip used to be raw
            `dep_kind: dep_value` strings, which told him it was blocked on
            something but not what. It is now one plain sentence per dependency. */}
        <td className="px-2 py-1.5">
          {d.deps.length > 0 ? (
            <span
              className="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-md text-[10px] font-bold bg-amber-500/10 text-amber-600 border border-amber-500/25 whitespace-nowrap"
              title={depsSentence}
            >
              <Link2 size={9} />↑{d.deps.length}
            </span>
          ) : (
            <span className="text-[11px] text-text-muted">—</span>
          )}
        </td>
        {/* Row actions */}
        <td className="px-2 py-1.5">
          <div className="flex items-center justify-end gap-0.5">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onReuse(d);
              }}
              title="以此为模板新建派单(席位/标题/指令/优先级会被带入表单)"
              aria-label="以此为模板新建派单"
              className="p-1 rounded-md text-text-secondary hover:text-primary hover:bg-accent transition-colors"
            >
              <CopyPlus size={13} />
            </button>
            <ChevronRight
              size={13}
              className={`text-text-secondary transition-transform ${expanded ? 'rotate-90' : ''}`}
            />
          </div>
        </td>
      </tr>

      {/* In-place expansion: the full payload and provenance, no detail page. */}
      {expanded && (
        <tr className="border-b border-border/20">
          <td colSpan={COLUMNS.length} className="px-2 pb-3 pt-0">
            <div className="rounded-lg border border-border/40 bg-secondary/40 p-3 space-y-2">
              {note ? (
                <p className="text-[12px] leading-5 text-text-primary whitespace-pre-wrap break-words">
                  {note}
                </p>
              ) : (
                <p className="text-[12px] text-text-secondary">该派单没有文字型 payload。</p>
              )}
              <dl className="grid grid-cols-2 md:grid-cols-4 gap-x-4 gap-y-1 pt-1 border-t border-border/30">
                <Meta label="编号" value={d.id} mono />
                <Meta label="发出者" value={d.issuedBy} />
                <Meta label="项目" value={d.projectId} />
                <Meta label="状态" value={statusStyle(d.status).label} />
                <Meta label="发出时间" value={fmtTime(d.issuedAt)} />
                <Meta label="开始时间" value={fmtTime(d.startedAt)} />
                <Meta label="完成时间" value={fmtTime(d.completedAt)} />
                <Meta
                  label="依赖"
                  value={d.deps.length > 0 ? resolved.map(depSentence).join('\n') : '无'}
                />
              </dl>
            </div>
          </td>
        </tr>
      )}
    </>
  );
};

const Meta: React.FC<{ label: string; value: string; mono?: boolean }> = ({
  label,
  value,
  mono,
}) => (
  <div className="min-w-0">
    <dt className="text-[9px] font-black text-text-muted uppercase tracking-widest">{label}</dt>
    <dd
      className={`text-[11px] text-text-primary truncate ${mono ? 'font-mono' : ''}`}
      title={value}
    >
      {value}
    </dd>
  </div>
);

// ── View ────────────────────────────────────────────────────────────────────

const DispatchInboxView: React.FC = () => {
  const [dispatches, setDispatches] = useState<DispatchDto[]>([]);
  const [seatsDb, setSeatsDb] = useState<SeatDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Liveness. One shared 1s clock drives every relative timestamp on screen
  // (elapsed, 发出, 上次刷新), so they can never disagree with each other.
  const [now, setNow] = useState(() => Date.now());
  const [lastRefreshAt, setLastRefreshAt] = useState<number | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);

  // Change marking (S4): a status flip between polls is highlighted briefly so
  // the transition is observed rather than inferred from a silently replaced row.
  const lastRowsRef = useRef<DispatchDto[]>([]);
  const [flashes, setFlashes] = useState<Record<string, number>>({});

  // Seats come from the DB (cmd_list_seats) so their ids (seat-mira-001, …) match
  // dispatches.target_seat_id — NOT from the UI store, whose active project may be
  // mock data with different ids.
  //
  // Scoped to the selected project. `seats` has no project_id, so the backend
  // resolves this through project_role_bindings — meaning a seat bound in another
  // project is correctly absent here rather than silently present.
  const activeProjectId = useDataStore((s) => s.activeProjectId);

  useEffect(() => {
    if (!activeProjectId) {
      setSeatsDb([]);
      return;
    }
    api
      .listSeats(activeProjectId)
      .then(setSeatsDb)
      .catch(() => setSeatsDb([]));
  }, [activeProjectId]);

  const load = useCallback(async (silent = false) => {
    if (!silent) setLoading(true);
    setError(null);
    // No project → nothing to show. The old code deliberately omitted the project
    // arg so the backend would fall back to seatloom, which is exactly how this
    // inbox showed another project's dispatches and looked populated.
    if (!activeProjectId) {
      setDispatches([]);
      setError('no project selected');
      if (!silent) setLoading(false);
      return;
    }
    try {
      const rows = await api.listDispatches(activeProjectId);

      const before = new Map(lastRowsRef.current.map((r) => [r.id, r.status]));
      const changed: Record<string, number> = {};
      for (const r of rows) {
        const prev = before.get(r.id);
        if (prev !== undefined && prev !== r.status) changed[r.id] = Date.now();
      }
      lastRowsRef.current = rows;

      setDispatches(rows);
      setLastRefreshAt(Date.now());

      if (Object.keys(changed).length > 0) {
        setFlashes((f) => ({ ...f, ...changed }));
        window.setTimeout(() => {
          setFlashes((f) => {
            const next = { ...f };
            for (const id of Object.keys(changed)) delete next[id];
            return next;
          });
        }, 2600);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      if (!silent) setLoading(false);
    }
  }, [activeProjectId]);

  useEffect(() => {
    void load();
  }, [load]);

  // Live refresh: poll every 5s so dispatches flow ready -> in_flight -> done
  // in real time (e.g. while watching a seat pull its next task). Pausable —
  // the operator keeps their place, and 刷新 is always one click away.
  useEffect(() => {
    if (!autoRefresh) return;
    const timer = setInterval(() => void load(true), 5000);
    return () => clearInterval(timer);
  }, [autoRefresh, load]);

  // Only non-done dispatches have a live clock (done ones are timed from
  // started_at to completed_at, which is fixed). When the whole inbox is done
  // there is nothing left to tick, so the interval stands down.
  const hasLiveClock = useMemo(() => dispatches.some((d) => d.status !== 'done'), [dispatches]);

  useEffect(() => {
    if (!hasLiveClock) return;
    const t = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(t);
  }, [hasLiveClock]);

  const seatName = useCallback(
    (seatId: string): string => seatsDb.find((s) => s.id === seatId)?.name ?? seatId,
    [seatsDb],
  );

  // --- Filter / sort / group state ---
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [query, setQuery] = useState('');
  const [sortKey, setSortKey] = useState<SortKey>('seat');
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc');
  const [groupMode, setGroupMode] = useState<'seat' | 'flat'>('seat');
  const [showDone, setShowDone] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  // Per-status counts always describe the whole inbox, never the current
  // filter — the chips are the summary, the table is the selection.
  const counts = useMemo(() => {
    const c: Record<string, number> = { all: dispatches.length };
    for (const d of dispatches) c[d.status] = (c[d.status] ?? 0) + 1;
    return c;
  }, [dispatches]);

  const onSort = useCallback(
    (key: SortKey) => {
      if (key === sortKey) {
        setSortDir((v) => (v === 'asc' ? 'desc' : 'asc'));
        return;
      }
      const spec = SORT_SPECS.find((c) => c.key === key);
      setSortKey(key);
      setSortDir(spec?.defaultDir ?? 'asc');
    },
    [sortKey],
  );

  // Status filter + free-text search over title, payload text, seat and id.
  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return dispatches.filter((d) => {
      if (statusFilter !== 'all' && d.status !== statusFilter) return false;
      if (!q) return true;
      return (
        d.title.toLowerCase().includes(q) ||
        d.id.toLowerCase().includes(q) ||
        d.targetSeatId.toLowerCase().includes(q) ||
        seatName(d.targetSeatId).toLowerCase().includes(q) ||
        payloadNote(d.payload).toLowerCase().includes(q)
      );
    });
  }, [dispatches, statusFilter, query, seatName]);

  const sorted = useMemo(() => {
    const rows = visible.slice();
    rows.sort((a, b) => {
      let r = 0;
      switch (sortKey) {
        case 'seat':
          r = a.targetSeatId.localeCompare(b.targetSeatId);
          break;
        case 'status':
          r = statusRank(a.status) - statusRank(b.status);
          break;
        case 'priority':
          r = a.priority - b.priority;
          break;
        case 'elapsed':
          r = elapsedMs(a, now) - elapsedMs(b, now);
          break;
        case 'issued':
          r = new Date(a.issuedAt).getTime() - new Date(b.issuedAt).getTime();
          break;
      }
      // Stable, scheduler-shaped tiebreak: lifecycle order, then the routing
      // engine's own pull order (priority DESC, issued_at ASC), then id.
      if (r === 0) r = statusRank(a.status) - statusRank(b.status);
      if (r === 0) r = b.priority - a.priority;
      if (r === 0) r = new Date(a.issuedAt).getTime() - new Date(b.issuedAt).getTime();
      if (r === 0) r = a.id.localeCompare(b.id);
      return sortDir === 'asc' ? r : -r;
    });
    return rows;
  }, [visible, sortKey, sortDir, now]);

  // Done rows are the only ones that stop needing attention, so they are the
  // only ones that get folded away — unless the operator asked for them.
  const activeRows = useMemo(() => sorted.filter((d) => d.status !== 'done'), [sorted]);
  const doneRows = useMemo(() => sorted.filter((d) => d.status === 'done'), [sorted]);
  const doneOpen = statusFilter === 'done' || showDone;
  const mainRows = statusFilter === 'done' ? doneRows : activeRows;

  const groups = useMemo(() => {
    if (groupMode === 'flat') return [{ key: '__flat__', rows: mainRows }];
    const map = new Map<string, DispatchDto[]>();
    for (const r of mainRows) {
      const arr = map.get(r.targetSeatId);
      if (arr) arr.push(r);
      else map.set(r.targetSeatId, [r]);
    }
    // Insertion order = order of each group's best (first) row, so the seat
    // holding the most urgent dispatch heads the list.
    return Array.from(map.entries()).map(([seatId, rows]) => ({ key: seatId, rows }));
  }, [groupMode, mainRows]);

  // --- New-dispatch form (operator commands a seat from the UI) ---
  const formRef = useRef<HTMLDivElement>(null);
  const titleRef = useRef<HTMLInputElement>(null);
  const inboxRef = useRef<HTMLDivElement>(null);
  const seats = seatsDb;
  const [showNew, setShowNew] = useState(false);
  const [newSeat, setNewSeat] = useState('');
  const [newTitle, setNewTitle] = useState('');
  const [newInstr, setNewInstr] = useState('');
  const [newPrio, setNewPrio] = useState(5);
  const [submitting, setSubmitting] = useState(false);
  const [submitErr, setSubmitErr] = useState<string | null>(null);

  const canSubmit = newSeat && newTitle.trim() && newInstr.trim() && !submitting;

  const handleCreate = useCallback(async () => {
    if (!newSeat || !newTitle.trim() || !newInstr.trim()) return;
    setSubmitting(true);
    setSubmitErr(null);
    try {
      await api.enqueueDispatch(newSeat, newTitle.trim(), newInstr.trim(), newPrio);
      setNewTitle('');
      setNewInstr('');
      setShowNew(false);
      await load(true);
    } catch (e) {
      setSubmitErr(String(e));
    } finally {
      setSubmitting(false);
    }
  }, [newSeat, newTitle, newInstr, newPrio, load]);

  // S5's loop: a ready/blocked row can be re-issued without retyping it.
  const reuseDispatch = useCallback((d: DispatchDto) => {
    setNewSeat(d.targetSeatId);
    setNewTitle(d.title);
    setNewInstr(payloadNote(d.payload));
    setNewPrio(d.priority);
    setSubmitErr(null);
    setShowNew(true);
    requestAnimationFrame(() => {
      formRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      titleRef.current?.focus();
    });
  }, []);

  // 打回 a delivery spawns a rework dispatch; the acceptance card links straight
  // to it. The inbox search already matches dispatch ids, so "locating" is a
  // search plus a scroll — no routing, no second surface to keep in sync.
  const locateDispatch = useCallback((dispatchId: string) => {
    setStatusFilter('all');
    setQuery(dispatchId);
    requestAnimationFrame(() => {
      inboxRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });
  }, []);

  const sortLabel = sortKey ? LABEL_BY_SORT.get(sortKey) : undefined;
  const clearQuery = useCallback(() => setQuery(''), []);

  return (
    <div className="flex-1 flex flex-col h-full bg-background overflow-hidden">
      <div className="p-8 border-b border-border/40 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-black tracking-tight text-text-primary uppercase">
            派单收件箱
          </h2>
          <p className="text-xs text-text-secondary mt-1">
            每个席位的治理派单队列(G19)· 进行中 → 就绪 → 阻塞 → 完成
          </p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowNew((v) => !v)}
            className="flex items-center gap-2 px-3 py-2 rounded-xl border border-primary/60 bg-primary text-white hover:bg-primary/90 transition-all shadow-sm text-xs font-bold"
          >
            <Plus size={14} />
            新建派单
          </button>
          <button
            onClick={() => void load()}
            className="flex items-center gap-2 px-3 py-2 rounded-xl border border-border/60 bg-card text-text-secondary hover:bg-accent transition-all shadow-sm text-xs font-bold"
          >
            <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
            刷新
          </button>
        </div>
      </div>

      {showNew && (
        <div ref={formRef} className="px-8 py-5 border-b border-border/40 bg-secondary/40">
          <div className="max-w-5xl space-y-3">
            <div className="flex flex-wrap gap-3">
              <div className="space-y-1">
                <label className="block text-[10px] font-black text-text-secondary uppercase tracking-widest">
                  派给席位
                </label>
                <select
                  value={newSeat}
                  onChange={(e) => setNewSeat(e.target.value)}
                  className="px-3 py-1.5 bg-card border border-border/40 rounded-lg text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary min-w-[160px]"
                >
                  <option value="">选择席位…</option>
                  {seats.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name ?? s.id}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-1">
                <label className="block text-[10px] font-black text-text-secondary uppercase tracking-widest">
                  优先级
                </label>
                <input
                  type="number"
                  value={newPrio}
                  onChange={(e) => setNewPrio(Number(e.target.value))}
                  className="px-3 py-1.5 bg-card border border-border/40 rounded-lg text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary w-24"
                />
              </div>
              <div className="space-y-1 flex-1 min-w-[220px]">
                <label className="block text-[10px] font-black text-text-secondary uppercase tracking-widest">
                  标题(一行摘要)
                </label>
                <input
                  ref={titleRef}
                  type="text"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  placeholder="例如:为派单页加失败态徽章"
                  className="w-full px-3 py-1.5 bg-card border border-border/40 rounded-lg text-xs focus:outline-none focus:ring-1 focus:ring-primary"
                />
              </div>
            </div>
            <div className="space-y-1">
              <label className="block text-[10px] font-black text-text-secondary uppercase tracking-widest">
                指令(payload — 席位将照此执行)
              </label>
              <textarea
                value={newInstr}
                onChange={(e) => setNewInstr(e.target.value)}
                rows={3}
                placeholder="把席位需要的全部上下文写在这里(自足:别依赖它当前分支没有的文件)。"
                className="w-full px-3 py-2 bg-card border border-border/40 rounded-lg text-xs focus:outline-none focus:ring-1 focus:ring-primary resize-y"
              />
            </div>
            {submitErr && (
              <div className="flex items-center gap-2 text-xs font-bold text-red-600">
                <AlertCircle size={14} /> 派单失败:{submitErr}
              </div>
            )}
            <div className="flex items-center gap-2">
              <button
                onClick={() => void handleCreate()}
                disabled={!canSubmit}
                className="flex items-center gap-2 px-4 py-2 rounded-xl bg-primary text-white text-xs font-bold shadow-sm transition-all disabled:opacity-40 hover:bg-primary/90"
              >
                <Send size={14} />
                {submitting ? '派发中…' : '派发'}
              </button>
              <button
                onClick={() => setShowNew(false)}
                className="px-4 py-2 rounded-xl border border-border/60 bg-card text-text-secondary text-xs font-bold hover:bg-accent"
              >
                取消
              </button>
              <span className="text-[11px] text-text-secondary">
                派发后会即时唤醒该席位(若它空闲)并出现在下方队列。
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Toolbar: status filter + search, then grouping / liveness / sort state.
          Sticky so the controls and the sort header never scroll away. */}
      <div className="px-8 pt-4 pb-3 border-b border-border/30 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 sticky top-0 z-10 space-y-2.5">
        <div className="flex flex-wrap items-center gap-2">
          {STATUS_FILTERS.map((f) => {
            const n = counts[f.key] ?? 0;
            const active = statusFilter === f.key;
            return (
              <button
                key={f.key}
                onClick={() => setStatusFilter(f.key)}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-bold border transition-all ${
                  active
                    ? 'bg-primary text-white border-primary'
                    : 'bg-card text-text-secondary border-border/50 hover:bg-accent'
                }`}
              >
                {f.label}
                <span
                  className={`text-[10px] px-1.5 rounded-full ${active ? 'bg-white/25' : 'bg-secondary'}`}
                >
                  {n}
                </span>
              </button>
            );
          })}
          <div className="relative group ml-auto w-full sm:w-64">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-text-secondary group-focus-within:text-primary transition-colors" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="搜索标题 / 指令 / 席位 / 编号…"
              className="w-full pl-8 pr-3 py-1.5 bg-secondary/60 border border-border/40 rounded-xl text-xs font-bold focus:outline-none focus:ring-1 focus:ring-primary focus:bg-card transition-all shadow-sm"
            />
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
          <div
            className="flex items-center p-0.5 rounded-lg border border-border/50 bg-card"
            role="group"
            aria-label="分组方式"
          >
            {(
              [
                { key: 'seat', label: '按席位' },
                { key: 'flat', label: '全局平铺' },
              ] as const
            ).map((m) => (
              <button
                key={m.key}
                onClick={() => setGroupMode(m.key)}
                className={`px-2.5 py-1 rounded-md text-[11px] font-bold transition-all ${
                  groupMode === m.key
                    ? 'bg-primary text-white shadow-sm'
                    : 'text-text-secondary hover:bg-accent'
                }`}
              >
                {m.label}
              </button>
            ))}
          </div>

          <span className="text-[11px] text-text-secondary">
            共 <span className="font-black text-text-primary">{visible.length}</span> 项
            {sortLabel && (
              <>
                {' '}
                · 按{sortLabel}
                {sortDir === 'asc' ? '升序' : '降序'}
              </>
            )}
          </span>

          <div className="ml-auto flex items-center gap-2">
            <span className="flex items-center gap-1.5 text-[10px] font-bold text-text-secondary">
              <span
                className={`w-1.5 h-1.5 rounded-full ${autoRefresh ? 'bg-emerald-500 animate-pulse' : 'bg-border'}`}
              />
              {autoRefresh ? '自动刷新 5s' : '自动刷新已暂停'}
              {lastRefreshAt !== null && (
                <span className="font-mono">· 上次 {fmtClock(lastRefreshAt)}</span>
              )}
            </span>
            <button
              onClick={() => setAutoRefresh((v) => !v)}
              title={autoRefresh ? '暂停 5s 自动刷新(仍可手动刷新)' : '恢复 5s 自动刷新'}
              aria-label={autoRefresh ? '暂停自动刷新' : '恢复自动刷新'}
              className="p-1.5 rounded-lg border border-border/60 bg-card text-text-secondary hover:bg-accent transition-all shadow-sm"
            >
              {autoRefresh ? <Pause size={12} /> : <Play size={12} />}
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto" ref={inboxRef}>
        <div className="px-8 py-5">
          <div className="max-w-6xl space-y-10">
            {error ? (
              <div className="flex items-center gap-2 text-sm font-bold text-red-600">
                <AlertCircle size={16} /> 加载派单失败:{error}
              </div>
            ) : mainRows.length === 0 && doneRows.length === 0 ? (
              <EmptyState
                dispatches={dispatches.length}
                hasQuery={query.trim().length > 0}
                hasStatusFilter={statusFilter !== 'all'}
                onClearQuery={clearQuery}
              />
            ) : (
              <div className="max-w-6xl space-y-6">
                {mainRows.length === 0 ? (
                  // Everything that matched is folded into 已完成.
                  <div className="flex flex-col items-center justify-center py-14 text-center">
                    <div className="p-4 bg-secondary rounded-full border border-border/40 text-text-secondary mb-4 opacity-20">
                      <Inbox className="w-8 h-8" />
                    </div>
                    <p className="text-sm font-bold text-text-secondary">无待处理的派单</p>
                    <p className="text-xs text-text-secondary mt-1 opacity-60">
                      {doneRows.length} 项已完成已折叠 — 它们不再需要操作者注意力
                    </p>
                    <button
                      onClick={() => setShowDone(true)}
                      className="mt-4 text-xs font-black text-primary uppercase tracking-widest hover:underline"
                    >
                      显示已完成({doneRows.length})
                    </button>
                  </div>
                ) : (
                  groups.map((g) => (
                    <section
                      key={g.key}
                      className="rounded-xl border border-border/40 overflow-hidden bg-card"
                    >
                      {groupMode === 'seat' ? (
                        <SeatGroupHeader seatId={g.key} rows={g.rows} seatName={seatName} />
                      ) : (
                        <div className="flex items-center gap-2 px-4 py-2 bg-secondary/40 border-b border-border/30">
                          <span className="text-[10px] font-black text-text-secondary uppercase tracking-widest">
                            全部席位
                          </span>
                          <span className="text-[11px] text-text-secondary">
                            {g.rows.length} 项 · 跨席位按{sortLabel ?? '默认'}排序
                          </span>
                        </div>
                      )}
                      <table className="w-full table-fixed border-collapse">
                        <colgroup>
                          {COL_WIDTHS.map((w, i) => (
                            <col key={i} className={w} />
                          ))}
                        </colgroup>
                        <TableHead sortKey={sortKey} sortDir={sortDir} onSort={onSort} />
                        <tbody>
                          {g.rows.map((d) => (
                            <DispatchRow
                              key={d.id}
                              d={d}
                              seatLabel={seatName(d.targetSeatId)}
                              now={now}
                              expanded={expandedId === d.id}
                              flash={flashes[d.id] !== undefined}
                              onToggle={(id) => setExpandedId((v) => (v === id ? null : id))}
                              onReuse={reuseDispatch}
                            />
                          ))}
                        </tbody>
                      </table>
                    </section>
                  ))
                )}

                {/* Completed work: one fold, not a wall of grey rows. */}
                {doneRows.length > 0 && statusFilter !== 'done' && (
                  <section className="rounded-xl border border-border/40 overflow-hidden bg-card">
                    <button
                      onClick={() => setShowDone((v) => !v)}
                      aria-expanded={doneOpen}
                      className="w-full flex items-center gap-2 px-4 py-2 bg-secondary/40 hover:bg-secondary/60 transition-colors text-left"
                    >
                      <ChevronRight
                        size={13}
                        className={`text-text-secondary transition-transform ${doneOpen ? 'rotate-90' : ''}`}
                      />
                      <span className="text-[10px] font-black text-text-secondary uppercase tracking-widest">
                        已完成
                      </span>
                      <span className="text-[10px] font-bold px-1.5 rounded-full bg-secondary text-text-secondary">
                        {doneRows.length}
                      </span>
                      <span className="ml-auto text-[10px] text-text-secondary">默认折叠</span>
                    </button>
                    {doneOpen && (
                      <table className="w-full table-fixed border-collapse">
                        <colgroup>
                          {COL_WIDTHS.map((w, i) => (
                            <col key={i} className={w} />
                          ))}
                        </colgroup>
                        <TableHead sortKey={sortKey} sortDir={sortDir} onSort={onSort} />
                        <tbody>
                          {doneRows.map((d) => (
                            <DispatchRow
                              key={d.id}
                              d={d}
                              seatLabel={seatName(d.targetSeatId)}
                              now={now}
                              expanded={expandedId === d.id}
                              flash={flashes[d.id] !== undefined}
                              onToggle={(id) => setExpandedId((v) => (v === id ? null : id))}
                              onReuse={reuseDispatch}
                            />
                          ))}
                        </tbody>
                      </table>
                    )}
                  </section>
                )}
              </div>
            )}

            {/* D2: the other half of the loop — delivered + Flux-verified work
                waiting on the user's 接受/打回. Same page, same 5s cadence. */}
            <AcceptanceSection
              seatName={seatName}
              live={autoRefresh}
              onLocateDispatch={locateDispatch}
            />

            {/* 020: the THIRD question this page answers — "which way do I rule
                on this?" A different object from an acceptance (which asks "do I
                take this delivered work?" and is about a dispatch), so it is a
                separate section with a separate count. Merging the two would
                make the operator's number wrong, which is the defect 020 exists
                to fix. */}
            <OperatorDecisionSection live={autoRefresh} />
          </div>
        </div>
      </div>
    </div>
  );
};

// S1: the group header is a triage line, not a label. Seat + its per-status
// counts answer "does this seat need me right now?" without reading a row.
const SeatGroupHeader: React.FC<{
  seatId: string;
  rows: DispatchDto[];
  seatName: (id: string) => string;
}> = ({ seatId, rows, seatName }) => {
  const label = seatName(seatId);
  const breakdown = useMemo(() => {
    const c = new Map<string, number>();
    for (const r of rows) c.set(r.status, (c.get(r.status) ?? 0) + 1);
    return Array.from(c.entries())
      .map(([key, n]) => ({ key, n }))
      .sort((a, b) => statusRank(a.key) - statusRank(b.key));
  }, [rows]);

  return (
    <div className="flex items-center gap-2 px-4 py-2 bg-secondary/40 border-b border-border/30">
      <SeatAvatar label={label} id={seatId} size="md" />
      <h3 className="text-[13px] font-black text-text-primary truncate">{label}</h3>
      <span className="font-mono text-[10px] text-text-muted truncate hidden lg:block">
        {seatId}
      </span>
      <div className="ml-auto flex items-center gap-2">
        {breakdown.map((b) => {
          const st = statusStyle(b.key);
          return (
            <span
              key={b.key}
              className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[10px] font-bold border whitespace-nowrap ${st.cls}`}
              title={`${st.label} ${b.n} 项`}
            >
              <span className={`w-1 h-1 rounded-full ${st.dot}`} />
              {st.label} {b.n}
            </span>
          );
        })}
        <span className="text-[10px] font-bold text-text-secondary tabular-nums">
          {rows.length} 项
        </span>
      </div>
    </div>
  );
};

// Empty states distinguish the three ways a table can be empty — nothing has
// ever been dispatched, the search matched nothing, or the status filter is
// empty — because each needs a different next action from the operator.
const EmptyState: React.FC<{
  dispatches: number;
  hasQuery: boolean;
  hasStatusFilter: boolean;
  onClearQuery: () => void;
}> = ({ dispatches, hasQuery, hasStatusFilter, onClearQuery }) => {
  const nothingAtAll = dispatches === 0;
  const headline = nothingAtAll
    ? '暂无派单'
    : hasQuery
      ? '没有匹配的派单'
      : hasStatusFilter
        ? '该状态下暂无派单'
        : '暂无派单';
  const hint = nothingAtAll
    ? '点右上「新建派单」把任务派给某个席位'
    : hasQuery
      ? '换个关键词,或清空搜索框'
      : '切换到「全部」查看其他状态的派单';

  return (
    <div className="flex flex-col items-center justify-center py-20 text-center">
      <div className="p-4 bg-secondary rounded-full border border-border/40 text-text-secondary mb-4 opacity-20">
        {nothingAtAll ? <Send className="w-8 h-8" /> : <Inbox className="w-8 h-8" />}
      </div>
      <p className="text-sm font-bold text-text-secondary">{headline}</p>
      <p className="text-xs text-text-secondary mt-1 opacity-60">{hint}</p>
      {hasQuery && (
        <button
          onClick={onClearQuery}
          className="mt-4 text-xs font-black text-primary uppercase tracking-widest hover:underline"
        >
          清空搜索
        </button>
      )}
    </div>
  );
};

export default DispatchInboxView;
