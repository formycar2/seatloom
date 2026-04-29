import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ProjectUIState } from '../types';

interface AppState {
  projectUIStates: Record<string, ProjectUIState>;
  navCollapsed: boolean;

  // Actions
  updateProjectUIState: (projectId: string, updates: Partial<ProjectUIState>) => void;
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

      updateProjectUIState: (projectId, updates) => set((state) => ({
        projectUIStates: {
          ...state.projectUIStates,
          [projectId]: {
            ...(state.projectUIStates[projectId] || DEFAULT_UI_STATE),
            ...updates,
          },
        },
      })),

      setNavCollapsed: (collapsed) => set({ navCollapsed: collapsed }),
      toggleNav: () => set((state) => ({ navCollapsed: !state.navCollapsed })),
    }),
    {
      name: 'seatloom-app-store',
    }
  )
);
