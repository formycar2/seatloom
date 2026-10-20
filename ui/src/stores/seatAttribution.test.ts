// Seats are attributed by ROLE BINDING, not to every project.
//
// The defect (2026-10-16, Mr. Zhang): QL appeared under the SeatLoom project. The
// hydration loop did `seatsDto.map(...)` inside the per-project loop, so every project
// carried every seat. Its comment said it was a v0.1 expedient to make the V1 shell
// render; what it actually did was assert a membership the data contradicts.
//
// These tests pin the filter through the store's own hydration path, because the bug
// lived in the store and a component-level test would have missed it.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { act } from 'react';

// The api layer is mocked at the module boundary, exactly as the store consumes it.
// The bindings are the ones measured on live: seatloom binds 6, ql_delivery binds 1,
// vela binds nowhere.
const state = vi.hoisted(() => ({
  projects: [
    {
      id: 'seatloom',
      name: 'SeatLoom',
      createdAt: '',
      workerBudgetTokens: 0,
      supervisorBudgetTokens: 0,
    },
    {
      id: 'ql_delivery',
      name: 'QL',
      createdAt: '',
      workerBudgetTokens: 0,
      supervisorBudgetTokens: 0,
    },
    {
      id: 'ThoughtOnly',
      name: 'ThoughtOnly',
      createdAt: '',
      workerBudgetTokens: 0,
      supervisorBudgetTokens: 0,
    },
  ],
  seats: [
    {
      id: 'seat-aegis-001',
      name: 'aegis',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-lyra-001',
      name: 'lyra',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-mira-001',
      name: 'mira',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-nimbus-001',
      name: 'nimbus',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-onyx-001',
      name: 'onyx',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-flux-001',
      name: 'flux',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-ql-001',
      name: 'ql',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
    {
      id: 'seat-vela-001',
      name: 'vela',
      capabilityTags: [],
      status: 'active',
      createdAt: '',
      defaultHarness: null,
      defaultAgent: null,
      defaultModel: null,
    },
  ],
  bindings: {} as Record<string, Array<{ seatId: string; role: string }>>,
}));

vi.mock('../lib/api', () => ({
  isTauri: () => true,
  api: {
    listProjects: async () => state.projects,
    listSeats: async () => state.seats,
    listRoleBindings: async (projectId?: string) => state.bindings[projectId ?? ''] ?? [],
    listWorkitems: async () => [],
    listHandoffs: async () => [],
    listArtifacts: async () => [],
    listDocuments: async () => [],
    listSessions: async () => [],
    listEvents: async () => [],
  },
}));

import { useDataStore } from './useDataStore';

const seatNames = (projectId: string) =>
  (useDataStore.getState().projectData[projectId]?.seats ?? []).map((s) => s.name).sort();

const unboundNames = (projectId: string) =>
  (useDataStore.getState().projectData[projectId]?.unboundSeats ?? []).map((s) => s.name).sort();

describe('seats are attributed by role binding', () => {
  beforeEach(() => {
    state.bindings = {
      seatloom: [
        { seatId: 'seat-aegis-001', role: 'supervisor' },
        { seatId: 'seat-flux-001', role: 'verifier' },
        { seatId: 'seat-lyra-001', role: 'product_owner' },
        { seatId: 'seat-mira-001', role: 'designer' },
        { seatId: 'seat-nimbus-001', role: 'architect' },
        { seatId: 'seat-onyx-001', role: 'data_engineer' },
      ],
      ql_delivery: [{ seatId: 'seat-ql-001', role: 'worker' }],
      // ThoughtOnly: none — like the 37 historical-corpus projects on live.
    };
  });

  it('the SeatLoom project carries exactly its six bound seats, and NOT ql', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    expect(seatNames('seatloom')).toEqual(['aegis', 'flux', 'lyra', 'mira', 'nimbus', 'onyx']);
    expect(seatNames('seatloom')).not.toContain('ql');
    expect(seatNames('seatloom')).not.toContain('vela');
  });

  it('ql_delivery carries exactly ql', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    expect(seatNames('ql_delivery')).toEqual(['ql']);
  });

  it('a project with no bindings carries no seats — and says which seats are unbound', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    expect(seatNames('ThoughtOnly')).toEqual([]);
    // The truth is carried so the UI can explain the empty list rather than render a
    // `席位 (0)` that reads as broken.
    expect(unboundNames('ThoughtOnly')).toEqual([
      'aegis',
      'flux',
      'lyra',
      'mira',
      'nimbus',
      'onyx',
      'ql',
      'vela',
    ]);
  });

  it('the bound seats still get their per-project role, not a blank one', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    const mira = useDataStore
      .getState()
      .projectData['seatloom'].seats.find((s) => s.name === 'mira');
    // seatFromDto maps the backend role through coerceRole: 'designer' -> its display label.
    expect(mira?.role).toBe('Designer');
  });

  it('an unbound seat is carried WITHOUT a binding role, so it cannot claim one it does not have', async () => {
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
    const vela = useDataStore
      .getState()
      .projectData['seatloom'].unboundSeats?.find((s) => s.name === 'vela');
    expect(vela).toBeDefined();
    // seatFromDto(undefined) coerces a missing role to { Custom: 'Unknown' } — the mapper's
    // own way of saying "no binding", never a guess at a real role.
    expect(vela?.role).toEqual({ Custom: 'Unknown' });
  });
});

// ── Switching projects must filter, not just re-label ─────────────────────────
//
// Mr. Zhang's 2026-10-17 bug: the project list showed many projects, but every
// module (live seats, dashboard, dispatches, work items, sessions, handoffs)
// showed the same data whichever project was selected. The rendering was already
// per-project (`projectData[activeProjectId]`); what was global was the CONTENT —
// the hydration loop put every row under every project.

describe('every list that carries project_id is filtered by it', () => {
  beforeEach(() => {
    state.projects = [
      {
        id: 'seatloom',
        name: 'SeatLoom',
        createdAt: '',
        workerBudgetTokens: 0,
        supervisorBudgetTokens: 0,
      },
      {
        id: 'ql_delivery',
        name: 'ql_delivery',
        createdAt: '',
        workerBudgetTokens: 0,
        supervisorBudgetTokens: 0,
      },
    ];
    state.seats = [];
    state.bindings = {
      seatloom: [
        { seatId: 'seat-mira-001', role: 'designer' },
        { seatId: 'seat-ql-001', role: 'worker' },
      ],
      ql_delivery: [{ seatId: 'seat-ql-001', role: 'worker' }],
    };
  });

  const hydrate = async () => {
    // The per-kind lists are installed by each test through vi.spyOn on the api
    // mock before calling this, so nothing is patched here.
    await act(async () => {
      await useDataStore.getState().hydrateFromBackend();
    });
  };

  it('work items, handoffs, sessions and events land under their OWN project only', async () => {
    // The mock returns [] for these; install project-tagged rows instead.
    const wi = (id: string, projectId: string) => ({
      id,
      projectId,
      title: id,
      status: 'Active',
      priority: 'Normal',
      ownerSeatId: null,
      acceptanceCriteria: [],
      createdAt: '',
      updatedAt: '',
    });
    const ho = (id: string, projectId: string) => ({
      id,
      projectId,
      purpose: id,
      status: 'Sent',
      requiredReceipt: false,
      fromRef: 'seat-aegis-001',
      toRef: 'seat-mira-001',
      createdAt: '',
      sentAt: null,
    });
    const se = (id: string, projectId: string) => ({
      id,
      projectId,
      seatId: 'seat-mira-001',
      runtime: 'Stepcode',
      status: 'Running',
      nativeSessionId: null,
      workspacePath: null,
      branch: null,
      createdAt: '',
    });
    const ev = (id: string, projectId: string) => ({
      id,
      eventType: 'SessionStarted',
      occurredAt: '2026-10-17T00:00:00Z',
      actorRef: 'seat-mira-001',
      payload: {},
      createdAt: '',
      projectId,
    });

    // Two rows per kind, one per project.
    const mod = await import('../lib/api');
    const spy = vi
      .spyOn(mod.api, 'listWorkitems')
      .mockResolvedValue([wi('wi-s', 'seatloom'), wi('wi-q', 'ql_delivery')] as never);
    const spyHo = vi
      .spyOn(mod.api, 'listHandoffs')
      .mockResolvedValue([ho('ho-s', 'seatloom'), ho('ho-q', 'ql_delivery')] as never);
    const spySe = vi
      .spyOn(mod.api, 'listSessions')
      .mockResolvedValue([se('se-s', 'seatloom'), se('se-q', 'ql_delivery')] as never);
    const spyEv = vi
      .spyOn(mod.api, 'listEvents')
      .mockResolvedValue([ev('ev-s', 'seatloom'), ev('ev-q', 'ql_delivery')] as never);

    try {
      await hydrate();
    } finally {
      spy.mockRestore();
      spyHo.mockRestore();
      spySe.mockRestore();
      spyEv.mockRestore();
    }

    const s = useDataStore.getState().projectData['seatloom'];
    const q = useDataStore.getState().projectData['ql_delivery'];
    expect(s.workItems.map((w) => w.id)).toEqual(['wi-s']);
    expect(q.workItems.map((w) => w.id)).toEqual(['wi-q']);
    expect(s.handoffs.map((h) => h.id)).toEqual(['ho-s']);
    expect(q.handoffs.map((h) => h.id)).toEqual(['ho-q']);
    expect(s.sessions.map((x) => x.id)).toEqual(['se-s']);
    expect(q.sessions.map((x) => x.id)).toEqual(['se-q']);
    // The V1 event model spells it `event_id`, not `id`.
    expect(s.events.map((x) => x.event_id)).toEqual(['ev-s']);
    expect(q.events.map((x) => x.event_id)).toEqual(['ev-q']);
  });

  it('documents are filtered too, and the runtime artifacts are NOT — and that is stated', async () => {
    // ArtifactDto carries no projectId, so the runtime rows cannot be filtered.
    // The test pins the asymmetry so nobody later reads it as an oversight.
    const mod = await import('../lib/api');
    const spyDoc = vi.spyOn(mod.api, 'listDocuments').mockResolvedValue([
      {
        id: 'doc-s',
        projectId: 'seatloom',
        title: 's',
        template: 'T5',
        subtype: 'delivery',
        status: 'active',
        createdAt: '',
        bodyDigest: null,
        updatedAt: null,
      },
      {
        id: 'doc-q',
        projectId: 'ql_delivery',
        title: 'q',
        template: 'T5',
        subtype: 'delivery',
        status: 'active',
        createdAt: '',
        bodyDigest: null,
        updatedAt: null,
      },
    ] as never);
    const spyArt = vi.spyOn(mod.api, 'listArtifacts').mockResolvedValue([
      {
        id: 'art-global',
        title: 'global',
        template: 'T5',
        subtype: 'x',
        status: 'active',
        createdAt: '',
        bodyDigest: null,
      },
    ] as never);
    try {
      await hydrate();
    } finally {
      spyDoc.mockRestore();
      spyArt.mockRestore();
    }
    const s = useDataStore.getState().projectData['seatloom'];
    const q = useDataStore.getState().projectData['ql_delivery'];
    // Documents follow the project…
    expect(s.artifacts.map((a) => a.id)).toContain('doc-s');
    expect(s.artifacts.map((a) => a.id)).not.toContain('doc-q');
    expect(q.artifacts.map((a) => a.id)).toContain('doc-q');
    // …and the runtime artifact, having no project_id, appears under both. That is
    // the documented gap, not a bug this test pretends is fixed.
    expect(s.artifacts.map((a) => a.id)).toContain('art-global');
    expect(q.artifacts.map((a) => a.id)).toContain('art-global');
  });
});
