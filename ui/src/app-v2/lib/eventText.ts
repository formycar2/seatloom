// eventText — turn a canonical ledger event into one line a person can read.
//
// Why this file exists (defect, fixed 2026-10-16): `useSupervisorData`'s
// `eventToMessage` recognised 22 event types. The database's vocabulary has 28, and
// the 16 it did not recognise all fell through to
//
//     default: content = `• ${etype}`
//
// so 79 % of the rows in the Aegis Supervisor column rendered as a bare English
// type name — real data, unreadable. The operator's read was that the panel looked
// like a mockup. It was not a mockup; it was a translation gap.
//
// The rules this file is written to:
//
// 1. **说人话,全中文。** No line's content is the event type name.
// 2. **带真正有意义的字段。** A dispatch line says which work, to whom, what
//    status. An acceptance line says what is being verified. A memory line says
//    the memory's title and its topic. Never just an id — the operator does not
//    read ids.
// 3. **不许编。** Every field shown comes from the event's own payload or refs, or
//    from a read the caller actually performed and handed in via
//    [`EventTextContext`]. When a lookup misses, the line says so. This project has
//    already paid for something that looked right and was a guess.
// 4. **The emoji prefix style is intentional** (the original comment explains it:
//    the V2 MessageBubble needs a populated `card` for non-text types, and the
//    prefix carries the category without card chrome). Preserved exactly.
// 5. **The default branch says something useful.** An unrecognised type renders
//    its name AND the payload keys it carried, so a future type degrades to
//    "here is what it contained" rather than to a bare word. That is what stops
//    this defect from recurring silently the next time the vocabulary grows.
//
// The 9 dead branches that were removed (SessionCompleted, SessionFailed,
// SessionInterrupted, HandoffDrafted, HandoffReceived, HandoffWorking,
// HandoffReturned, CheckpointCreated, PromptDetected) are not in the database's
// vocabulary at all — `query_ledger` rejects them by name, so they could never
// match. Keeping them made the switch look like it covered more than it did, which
// is the same failure in the opposite direction.

import type { CanonicalEventDto } from '../../lib/types-dto';

/** How long a quoted reason may run before it is cut, with the cut stated. */
const REASON_LIMIT = 300;
/** How long a quoted scenario may run before it is cut, with the cut stated. */
const SCENARIO_LIMIT = 120;

/** What a dispatch row gives us, resolved once per panel load. */
export interface DispatchFact {
  title: string;
  targetSeatId: string;
  status: string;
}

/** What a memory card gives us, resolved once per panel load. */
export interface MemoryFact {
  title: string;
  topicId: string;
}

/**
 * Everything a line may resolve against. Every map is optional: a missing map (or
 * a miss inside one) makes the line say what it could not resolve, never guess.
 */
export interface EventTextContext {
  /** dispatch id -> its title / target seat / status. */
  dispatches?: Map<string, DispatchFact>;
  /** memory id -> its title / topic. */
  memories?: Map<string, MemoryFact>;
  /** backend seat id -> display name. */
  seatNames?: Map<string, string>;
}

/** A non-empty string, or null. Anything else (number, object, "") is null. */
function str(v: unknown): string | null {
  return typeof v === 'string' && v.trim() !== '' ? v : null;
}

/** Cut `s` to `limit`, saying how long the original was when it had to cut. */
function quote(s: string, limit: number): string {
  if (s.length <= limit) return s;
  return `${s.slice(0, limit)}…(原文 ${s.length} 字,此处截断;完整内容在账本里)`;
}

/**
 * The dispatch id an event is about, from its own payload.
 *
 * NOT from `evt.refs`: the DTO the UI receives (`CanonicalEventDto`) carries id,
 * type, occurredAt, actorRef and payload — the object refs are not on the wire,
 * so a line cannot be built from them. Where the payload has no dispatch id
 * (`AcceptanceGranted` carries only branch/base/merge_commit) the line says what
 * it does know instead of guessing the work from an id shape.
 */
function dispatchIdOf(payload: Record<string, unknown>): string | null {
  return str(payload.dispatch_id);
}

/** `seat:seat-mira-001` and `seat-mira-001` both -> `seat-mira-001`. */
function seatIdOf(actorRef: string | null | undefined): string | null {
  const raw = str(actorRef);
  if (!raw) return null;
  return raw.startsWith('seat:') ? raw.slice('seat:'.length) : raw;
}

/** A seat id as a display name. Falls back to the id itself — never to a guess. */
function seatLabel(ctx: EventTextContext | undefined, seatId: string | null): string {
  if (!seatId) return '(没记是谁)';
  if (seatId.toLowerCase().startsWith('human')) return '操作者';
  const named = ctx?.seatNames?.get(seatId);
  if (named) return named;
  return seatId;
}

/** "《title》" when the title resolved; an honest note when it did not. */
function workLabel(ctx: EventTextContext | undefined, dispatchId: string | null): string {
  if (!dispatchId) return '(这条没记是哪个活)';
  const fact = ctx?.dispatches?.get(dispatchId);
  if (!fact) return `《${dispatchId}》(这个派单的标题没取到)`;
  return `《${fact.title}》`;
}

const STATUS_LABELS: Record<string, string> = {
  ready: '待认领',
  blocked: '被卡住',
  in_flight: '进行中',
  done: '已完成',
};

function statusLabel(status: string): string {
  return STATUS_LABELS[status] ?? status;
}

/**
 * One ledger event -> one line of Chinese.
 *
 * `ctx` is optional so callers that have not resolved anything still get honest
 * output rather than a crash.
 */
export function eventToText(evt: CanonicalEventDto, ctx?: EventTextContext): string {
  const payload = (evt.payload ?? {}) as Record<string, unknown>;
  const etype = evt.eventType;
  const dispatchId = dispatchIdOf(payload);
  const work = workLabel(ctx, dispatchId);
  const actor = seatIdOf(evt.actorRef);

  switch (etype) {
    // ── 派单的一生 ─────────────────────────────────────────────────────────
    case 'SeatDispatchIssued': {
      const fact = dispatchId ? ctx?.dispatches?.get(dispatchId) : undefined;
      const to = fact ? seatLabel(ctx, fact.targetSeatId) : '(派给谁没记)';
      const status = fact ? `,状态 ${statusLabel(fact.status)}` : '';
      return `🆕 派了新活 ${work} → ${to}${status}`;
    }
    case 'SeatDispatchDelivered':
      return `📬 活送到 ${seatLabel(ctx, actor)} 手上:${work}`;
    case 'SeatDispatchCompleted':
      return `✅ ${seatLabel(ctx, actor)} 做完了 ${work}`;
    case 'SeatDispatchCancelled':
      return `🚫 ${work} 取消了`;
    case 'SeatDispatchFailed': {
      const reason = str(payload.reason);
      const retries = typeof payload.retry_count === 'number' ? payload.retry_count : null;
      const max = typeof payload.max_retries === 'number' ? payload.max_retries : null;
      const exhausted = payload.retries_exhausted === true;
      const tail = exhausted
        ? `;重试次数用完了(${retries ?? '?'}/${max ?? '?'})`
        : retries != null && max != null
          ? `;重试 ${retries}/${max}`
          : '';
      return reason
        ? `🔴 ${work} 没做成:${quote(reason, REASON_LIMIT)}${tail}`
        : `🔴 ${work} 没做成(没记原因)${tail}`;
    }
    case 'SeatDispatchRetried': {
      const reason = str(payload.reason);
      const retries = typeof payload.retry_count === 'number' ? payload.retry_count : null;
      const max = typeof payload.max_retries === 'number' ? payload.max_retries : null;
      const budget = retries != null && max != null ? `(${retries}/${max})` : '';
      return reason
        ? `🔁 ${work} 打回去重做${budget}:${quote(reason, REASON_LIMIT)}`
        : `🔁 ${work} 打回去重做${budget}(没记原因)`;
    }

    // ── 验收卡 ─────────────────────────────────────────────────────────────
    case 'AcceptanceRequested': {
      const plan = (payload.acceptance_plan ?? {}) as Record<string, unknown>;
      const scenario = str(plan.scenario);
      const branch = str(payload.branch);
      const what = scenario ? quote(scenario, SCENARIO_LIMIT) : '(这张卡没写要验什么)';
      return `🧾 交了张验收卡等你看:${what}${branch ? `(分支 ${branch})` : ''}`;
    }
    case 'AcceptanceGranted': {
      const branch = str(payload.branch);
      const commit = str(payload.merge_commit);
      const where = branch ? `合进 ${branch}` : '(没记合到哪个分支)';
      const at = commit ? `,合并点 ${commit.slice(0, 12)}` : '';
      return `🤝 你点了接受,${work} ${where}${at}`;
    }
    case 'AcceptanceRejected': {
      const feedback = str(payload.feedback);
      const rework = str(payload.rework_dispatch_id);
      const tail = rework ? `;已另派一张重做的单(${rework})` : ';没有另派重做的单';
      return feedback
        ? `✋ 你打回了 ${work}:${quote(feedback, REASON_LIMIT)}${tail}`
        : `✋ 你打回了 ${work}(没写原因)${tail}`;
    }
    case 'AcceptanceCancelled': {
      const reason = str(payload.reason);
      return reason
        ? `🗑 ${work} 的验收卡作废了:${quote(reason, REASON_LIMIT)}`
        : `🗑 ${work} 的验收卡作废了(没记原因)`;
    }

    // ── 记忆 ───────────────────────────────────────────────────────────────
    case 'MemoryCreated': {
      const title = str(payload.title);
      const kind = str(payload.kind);
      const topic = str(payload.topic_id);
      const what = title ? `《${title}》` : '(没写标题)';
      const tail = [kind ? `类型 ${kind}` : null, topic ? `归到主题 ${topic}` : null]
        .filter(Boolean)
        .join(' · ');
      return `🧠 记了一条新记忆 ${what}${tail ? `(${tail})` : ''}`;
    }
    case 'MemoryRecalled': {
      const id = str(payload.memory_id);
      const fact = id ? ctx?.memories?.get(id) : undefined;
      const what = fact
        ? `《${fact.title}》`
        : id
          ? `(${id} 这条记忆的标题没取到)`
          : '(没记是哪条)';
      const heat = typeof payload.heat_delta === 'number' ? payload.heat_delta : null;
      const tail = heat != null ? `,被翻了 ${heat > 0 ? '+' : ''}${heat} 次` : '';
      return `🔍 翻到一条记忆 ${what}${tail}`;
    }
    case 'MemoryPromoted': {
      const from = str(payload.from_tier);
      const to = str(payload.to_tier);
      const id = str(payload.memory_id);
      const fact = id ? ctx?.memories?.get(id) : undefined;
      const what = fact
        ? `《${fact.title}》`
        : id
          ? `(${id} 这条记忆的标题没取到)`
          : '(没记是哪条)';
      return from && to
        ? `⬆️ ${what} 的冷热从 ${from} 变成 ${to}`
        : `⬆️ ${what} 的冷热变了(没记从什么变成什么)`;
    }
    case 'MemoryForgotten': {
      const id = str(payload.memory_id);
      const fact = id ? ctx?.memories?.get(id) : undefined;
      const what = fact
        ? `《${fact.title}》`
        : id
          ? `(${id} 这条记忆的标题没取到)`
          : '(没记是哪条)';
      const reason = str(payload.reason);
      return reason
        ? `🧹 忘掉了 ${what}:${quote(reason, REASON_LIMIT)}`
        : `🧹 忘掉了 ${what}(没记原因)`;
    }
    case 'MemoryResolved': {
      const id = str(payload.memory_id);
      const fact = id ? ctx?.memories?.get(id) : undefined;
      const what = fact
        ? `《${fact.title}》`
        : id
          ? `(${id} 这条记忆的标题没取到)`
          : '(没记是哪条)';
      const by = seatIdOf(str(payload.resolved_by));
      return `✔️ ${what} 结了${by ? `,${seatLabel(ctx, by)} 结的` : '(没记是谁结的)'}`;
    }

    // ── 主题 ───────────────────────────────────────────────────────────────
    case 'TopicOpened': {
      const title = str(payload.title);
      const parent = str(payload.parent_topic_id);
      return title
        ? `📂 开了个主题《${title}》${parent ? `,挂在 ${parent} 底下` : ''}`
        : '(开了个主题,但没记标题)';
    }

    // ── 原有分支,词表里确实存在 ────────────────────────────────────────────
    case 'SessionStarted':
      return `🟢 会话起来了 — ${str(payload.session_id) ?? '(没记会话号)'}`;
    case 'ArtifactCreated':
      return `📄 出了个产物 — ${str(payload.artifact_id) ?? str(payload.title) ?? '(没记是哪个)'}`;
    case 'HandoffSent':
    case 'HandoffAccepted':
    case 'HandoffCompleted':
      return `📤 ${etype} — ${str(payload.handoff_id) ?? '(没记是哪个)'}`;
    case 'WorkItemCreated':
      return `📋 开了个活 — ${str(payload.title) ?? str(payload.workitem_id) ?? '(没写标题)'}`;
    case 'WorkItemStatusChanged':
      return `📋 活的状态变成 ${str(payload.new_status) ?? '(没记新状态)'}(${str(payload.workitem_id) ?? '没记是哪个活'})`;
    case 'ReviewVerdictIssued':
      return `✅ 验完了:${str(payload.verdict) ?? '(没记结论)'} — ${str(payload.workitem_id) ?? '(没记是哪个活)'}`;
    case 'SeatDelegationIssued':
      return `🔁 ${seatLabel(ctx, seatIdOf(str(payload.from_seat)))} 把活委托给 ${seatLabel(ctx, seatIdOf(str(payload.to_seat)))}`;
    case 'SeatDelegationClosed':
      return `🔁 委托结束了 — ${str(payload.delegation_id) ?? '(没记是哪个)'}`;

    // 真人说过的话:原样透出,一个字都不改。这是验收卡里明确不许退化的那一项。
    //
    // 一条没有 content 的 SupervisorMessage 不是人说的话,是闸门声明 —— 它的
    // 有意义字段是 statement(闸门到底是什么条件、满足了没有)。2026-10-16 派单
    // 的依赖列第一次把这个形状摆到台面上:一条 -live 闸门事件被翻译成「这条是
    // 空的」,而它明明带着一整段 statement。有 content 就还是原样透出,一个字节
    // 都不动;没有 content 才退到 statement。
    case 'SupervisorMessage':
    case 'SeatResponse': {
      const content = str(payload.content);
      if (content) return content;
      const statement = str(payload.statement);
      if (statement) return `闸门声明:${statement}`;
      return '[这条是空的]';
    }

    default:
      // Not a bare type name: say what it was AND what it carried, so the next
      // type added to the vocabulary degrades to something readable.
      return `📌 ${etype}(这个类型还没有人话翻译;它带了这些字段:${Object.keys(payload).join(', ') || '什么都没有'})`;
  }
}

/** The payload keys a `default`-branch line reports, for tests. */
export function payloadKeys(evt: CanonicalEventDto): string[] {
  return Object.keys((evt.payload ?? {}) as Record<string, unknown>);
}
