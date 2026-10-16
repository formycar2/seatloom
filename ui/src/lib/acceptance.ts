// D2 acceptance loop — the boundary between this UI and Nimbus's commands.
//
// The card contract was agreed across lanes, but when this UI was written the
// commands were not on `track`, so the exact wire spelling could not be read off
// the Rust DTO. Two spellings are in play:
//
//   * the house convention — camelCase, because the backend serialises with
//     `#[serde(rename_all = \"camelCase\")]` (see the header of `types-dto.ts`);
//   * the dispatch that commissioned this lane, which spelled the same fields
//     snake_case (`dispatch_id`, `how_to_verify`, `flux_verdict`, …).
//
// Rather than bet the whole section on a guess, the boundary accepts both and
// normalises to the camelCase shape the rest of the app speaks. When Nimbus's
// DTO lands this keeps working unchanged; if it disagrees with the house
// convention it also keeps working. Flux's full-chain proof is where the actual
// spelling gets confirmed — this module only stops a spelling mismatch from
// presenting as a blank section.
//
// This is a wire-shape adapter, not a fixture: nothing here invents a value.
// A field that is absent on the wire stays absent (empty string / null) and the
// UI says so.

import type { AcceptanceCardDto, FluxVerdictDto } from './types-dto';

type Raw = Record<string, unknown>;

/** First present, non-null value among `keys`, in order. */
function pick<T>(raw: Raw, keys: readonly string[]): T | undefined {
  for (const k of keys) {
    const v = raw[k];
    if (v !== undefined && v !== null) return v as T;
  }
  return undefined;
}

function asString(raw: Raw, keys: readonly string[]): string {
  const v = pick<unknown>(raw, keys);
  return typeof v === 'string' ? v : '';
}

function asStringOrNull(raw: Raw, keys: readonly string[]): string | null {
  const v = pick<unknown>(raw, keys);
  return typeof v === 'string' ? v : null;
}

/** Camel-case and snake_case spellings of one logical field. */
const FIELD = {
  id: ['id'],
  dispatchId: ['dispatchId', 'dispatch_id'],
  dispatchTitle: ['dispatchTitle', 'dispatch_title'],
  seatId: ['seatId', 'seat_id'],
  branch: ['branch'],
  base: ['base'],
  howToVerify: ['howToVerify', 'how_to_verify'],
  status: ['status'],
  createdAt: ['createdAt', 'created_at'],
  mergeCommit: ['mergeCommit', 'merge_commit'],
  feedback: ['feedback'],
  reworkDispatchId: ['reworkDispatchId', 'rework_dispatch_id'],
  // flux_verdict: { ref_id, verdict, occurred_at }
  refId: ['refId', 'ref_id'],
  verdict: ['verdict'],
  occurredAt: ['occurredAt', 'occurred_at'],
} as const;

function normaliseVerdict(raw: Raw): FluxVerdictDto {
  const nested = pick<Raw>(raw, ['fluxVerdict', 'flux_verdict']);
  const source: Raw = nested && typeof nested === 'object' ? nested : raw;
  return {
    refId: asString(source, FIELD.refId),
    verdict: asString(source, FIELD.verdict),
    occurredAt: asString(source, FIELD.occurredAt),
  };
}

/**
 * One card, in the shape the UI renders. Unknown extra fields are dropped
 * rather than guessed at — if Nimbus adds something the section needs, it
 * belongs in `types-dto.ts` deliberately, not by accident.
 */
export function normaliseAcceptanceCard(input: unknown): AcceptanceCardDto | null {
  if (!input || typeof input !== 'object') return null;
  const raw = input as Raw;

  return {
    id: asString(raw, FIELD.id),
    dispatchId: asString(raw, FIELD.dispatchId),
    dispatchTitle: asString(raw, FIELD.dispatchTitle),
    seatId: asString(raw, FIELD.seatId),
    branch: asString(raw, FIELD.branch),
    base: asString(raw, FIELD.base),
    howToVerify: asString(raw, FIELD.howToVerify),
    fluxVerdict: normaliseVerdict(raw),
    status: asString(raw, FIELD.status),
    createdAt: asString(raw, FIELD.createdAt),
    mergeCommit: asStringOrNull(raw, FIELD.mergeCommit),
    feedback: asStringOrNull(raw, FIELD.feedback),
    reworkDispatchId: asStringOrNull(raw, FIELD.reworkDispatchId),
  };
}

/** A list response, in the shape the UI renders. Non-objects are skipped. */
export function normaliseAcceptanceCards(input: unknown): AcceptanceCardDto[] {
  if (!Array.isArray(input)) return [];
  return input.map(normaliseAcceptanceCard).filter((c): c is AcceptanceCardDto => c !== null);
}

// ── Presentation helpers ────────────────────────────────────────────────────

export const ACCEPTANCE_STATUS: Record<string, { label: string; cls: string }> = {
  awaiting_acceptance: {
    label: '待验收',
    cls: 'bg-amber-500/15 text-amber-600 border-amber-500/30',
  },
  accepted: { label: '已接受', cls: 'bg-emerald-500/15 text-emerald-600 border-emerald-500/30' },
  rejected: { label: '已打回', cls: 'bg-rose-500/15 text-rose-600 border-rose-500/30' },
  // Voiding is a terminal state, not "not decided yet". Missing this entry is what
  // made a voided card render under its raw status string — and, via isDecided,
  // get counted as waiting on the operator.
  cancelled: { label: '已作废', cls: 'bg-secondary text-text-secondary border-border/40' },
};

export function acceptanceStatusStyle(status: string): { label: string; cls: string } {
  return (
    ACCEPTANCE_STATUS[status] ?? {
      label: status || '未知状态',
      cls: 'bg-secondary text-text-secondary border-border/40',
    }
  );
}

/**
 * True once the card is settled — it shows a result, not actions.
 *
 * Four states, not two. `AcceptanceCardDto.status` is
 * 'pending' | 'accepted' | 'rejected' | 'cancelled' (see types-dto.ts), and
 * `cancel_acceptance` is terminal: a cancelled card never comes back for a
 * decision. Counting it as undecided is what made the operator's 待验收 read 7
 * when 5 were actually waiting on him.
 *
 * Cancelled cards are still rendered — in their own 已作废 area, with the reason —
 * so a voided card stays findable without inflating the number he must act on.
 */
export function isDecided(status: string): boolean {
  return status === 'accepted' || status === 'rejected' || status === 'cancelled';
}

/** A card that was voided rather than decided. Kept visible, never counted. */
export function isCancelled(status: string): boolean {
  return status === 'cancelled';
}

/** Flux's verdict as a badge. Unknown verdicts stay neutral, never green. */
export function fluxVerdictStyle(verdict: string): { label: string; cls: string } {
  const v = verdict.toLowerCase();
  if (v === 'pass' || v === 'passed' || v === 'green') {
    return { label: 'Flux 验绿', cls: 'bg-emerald-500/15 text-emerald-600 border-emerald-500/30' };
  }
  if (v === 'fail' || v === 'failed' || v === 'red') {
    return { label: 'Flux 验红', cls: 'bg-rose-500/15 text-rose-600 border-rose-500/30' };
  }
  return {
    label: verdict ? `Flux ${verdict}` : 'Flux 未判',
    cls: 'bg-secondary text-text-secondary border-border/40',
  };
}

/**
 * A merge that could not be applied surfaces as a rejected promise whose message
 * is git's own output. It is shown verbatim — the operator needs the conflict
 * hunks, not a paraphrase.
 */
export function looksLikeMergeConflict(message: string): boolean {
  return /conflict|CONFLICT|Automatic merge failed|not possible|fast-forward/i.test(message);
}
