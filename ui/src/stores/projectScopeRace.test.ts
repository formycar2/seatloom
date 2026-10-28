import { beforeEach, describe, expect, it, vi } from 'vitest';
import { boundMirrorNames } from '../lib/project-scope';
const pending = vi.hoisted(() => ({ reads: new Map<string, (rows: unknown[]) => void>() }));
vi.mock('../lib/api', () => ({
  isTauri: () => true,
  api: {
    listSeats: async () => [], listAllSeats: async () => [],
    listRoleBindings: async () => [],
    listWorkitems: (id: string) => new Promise((resolve) => pending.reads.set(id, resolve)),
    listHandoffs: async () => [], listSessions: async () => [],
    listArtifacts: async () => [], listEvents: async () => [], listDocuments: async () => [],
  },
}));
import { useDataStore } from './useDataStore';
const finish = async (id: string) => {
  pending.reads.get(id)!([]);
  await vi.waitFor(() => expect(useDataStore.getState().backendHydrated).toBe(true));
};
beforeEach(() => {
  pending.reads.clear();
  useDataStore.setState({ activeProjectId: null, projectData: {}, backendHydrated: false, backendHydrateError: null });
});
describe('combined project filtering', () => {
  it('late A responses do not publish after B has been selected and loaded', async () => {
    useDataStore.getState().setActiveProject('A');
    useDataStore.getState().setActiveProject('B');
    await finish('B');
    pending.reads.get('A')!([]);
    await new Promise((resolve) => setTimeout(resolve, 0));
    const s = useDataStore.getState();
    expect(s.activeProjectId).toBe('B');
    expect(s.projectData.B).toBeDefined();
    expect(s.projectData.A).toBeUndefined();
    expect(s.backendHydrated).toBe(true);
  });
  it('a switch marks data unresolved immediately rather than displaying cached numbers as live', () => {
    useDataStore.setState({ activeProjectId: 'A', backendHydrated: true });
    useDataStore.getState().setActiveProject('B');
    expect(useDataStore.getState().backendHydrated).toBe(false);
  });
  it('the actual mirror session set follows registered binding names, including Aegis', () => {
    const sessions = ['Aegis', 'Nimbus', 'QL'].map((seatName) => ({ seatName }));
    expect(boundMirrorNames(sessions, [{ name: 'aegis' }, { name: 'nimbus' }])).toEqual(sessions.slice(0, 2));
    expect(boundMirrorNames(sessions, [{ name: 'ql' }])).toEqual([sessions[2]]);
    expect(boundMirrorNames(sessions, [])).toEqual([]);
  });
});
