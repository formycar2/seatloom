import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ProjectUIState } from '../types';

interface AppState {
  projectUIStates: Record<string, ProjectUIState>;
  navCollapsed: boolean;
  /**
   * The last project the operator was in, persisted so a restart lands back in
   * it rather than on a default tab.
   *
   * 2026-10-20 (AEGIS-2026-10-20-ui-memory-and-desync-v1 item 1): the per-project
   * tab/selection state was already persisted by this store's `persist`, but the
   * active project id itself lived only in `App.tsx`'s `useState` — so a restart
   * lost the position even though the destination was on disk. Persisting the id
   * here (the same store, the same mechanism, one localStorage key) is the whole
   * fix; no new storage key is introduced.
   */
  activeProjectId: string | null;

  // Actions
  updateProjectUIState: (projectId: string, updates: Partial<ProjectUIState>) => void;
  setActiveProjectId: (projectId: string | null) => void;
  setNavCollapsed: (collapsed: boolean) => void;
  toggleNav: () => void;
}

const DEFAULT_UI_STATE: ProjectUIState = {
  activeTab: 'dashboard',
  selectedObjectId: null,
  filters: {},
};

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      projectUIStates: {},
      navCollapsed: false,
      // `null` = no recorded position yet; the app falls back to its normal
      // default rather than to a guessed project.
      activeProjectId: null,

      updateProjectUIState: (projectId, updates) =>
        set((state) => ({
          projectUIStates: {
            ...state.projectUIStates,
            [projectId]: {
              ...(state.projectUIStates[projectId] || DEFAULT_UI_STATE),
              ...updates,
            },
          },
        })),

      setActiveProjectId: (projectId) => set({ activeProjectId: projectId }),

      setNavCollapsed: (collapsed) => set({ navCollapsed: collapsed }),
      toggleNav: () => set((state) => ({ navCollapsed: !state.navCollapsed })),
    }),
    {
      name: 'seatloom-app-store',
    },
  ),
);
