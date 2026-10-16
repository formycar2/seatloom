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
