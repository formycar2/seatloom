// intent-dto.ts — the TypeScript mirror of plan_intent's return (crate seatloom-core::intent).
//
// Product contract: LYRA-2026-10-13-supervisor-intent-entry-contract-v1, slice 1.
// Field names are camelCase because the Rust structs serialise with
// `rename_all = "camelCase"`; the verb/object-type/dep-kind enums are kebab/snake_case
// exactly as the tool emits them. This is the wire the UI renders against — every
// suggestion-card field is one of these, filled by plan_intent from real reads.

/** dep_kind: event | artifact | dispatch (snake_case). */
export type DepKind = 'event' | 'artifact' | 'dispatch';

export interface SuggestedDependency {
  depKind: DepKind;
  depValue: string;
}

export interface PreconditionResolution {
  raw?: string | null;
  edge?: SuggestedDependency | null;
  matchedRule?: string | null;
  unresolvedReason?: string | null;
}

export interface EvidenceLink {
  label: string;
  source: string;
}

export interface BudgetFacts {
  projectWorkerBudgetTokens?: number | null;
  projectSupervisorBudgetTokens?: number | null;
  seatInFlightDispatches: number;
  statedHint?: string | null;
  note: string;
}

export interface ProposedObject {
  kind: string;
  title: string;
  payloadNote: string;
  suggestedDependencies: SuggestedDependency[];
}

export interface Ownership {
  seatId: string;
  seatName: string;
  role?: string | null;
  capabilityMatch: string[];
  authorityDocs: string[];
  constraints: string[];
  activeDelegationId?: string | null;
}

export interface SeatRationale {
  seatId: string;
  seatName: string;
  eligible: boolean;
  matchedCapabilities: string[];
  excludingConstraint?: string | null;
  role?: string | null;
  authorityDocs: string[];
  constraints: string[];
  inFlightDispatches: number;
  selected: boolean;
  source: string;
}

export interface ClarificationChoice {
  value: string;
  label: string;
  why: string;
}

export interface Clarification {
  ruleId: string;
  question: string;
  reason: string;
  choices: ClarificationChoice[];
}

export interface Proposal {
  intentRestated: string;
  proposedObject: ProposedObject;
  ownership: Ownership;
  evidence: EvidenceLink[];
  budget: BudgetFacts;
  impact: string[];
  routingRationale: SeatRationale[];
  precondition: PreconditionResolution;
  blockingClarification?: Clarification | null;
  actions: string[];
  actor: string;
}

/** The exact reads a why/evidence verb should make — never a composed answer. */
export interface Retrieval {
  [key: string]: unknown;
}

/** plan_intent's whole return, parsed. `outcome` picks which result object is present. */
export interface IntentPlanResult {
  contract: string;
  outcome: 'proposal' | 'clarification' | 'retrieval';
  wrote: boolean;
  proposal?: Proposal;
  clarification?: Clarification;
  retrieval?: Retrieval;
}
