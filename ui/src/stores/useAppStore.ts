import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { ProjectUIState } from '../types';
import { ThemePreset, THEME_PRESETS } from '../styles/theme';

interface AppState {
  projectUIStates: Record<string, ProjectUIState>;
  themePreset: ThemePreset;
  
  // Actions
  updateProjectUIState: (projectId: string, updates: Partial<ProjectUIState>) => void;
  setThemePreset: (preset: ThemePreset) => void;
}

const DEFAULT_UI_STATE: ProjectUIState = {
  activeTab: 'inbox',
  selectedObjectId: null,
  filters: {},
};

const VALID_PRESETS = THEME_PRESETS.map(p => p.id);

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      projectUIStates: {},
      themePreset: 'paper-ledger',

      updateProjectUIState: (projectId, updates) => set((state) => ({
        projectUIStates: {
          ...state.projectUIStates,
          [projectId]: {
            ...(state.projectUIStates[projectId] || DEFAULT_UI_STATE),
            ...updates,
          },
        },
      })),

      setThemePreset: (preset) => {
        set({ themePreset: preset });
        document.body.setAttribute('data-theme', preset);
      },
    }),
    {
      name: 'seatloom-app-store',
      onRehydrateStorage: () => (state) => {
        if (state) {
          // Normalize invalid preset to default
          if (!state.themePreset || !VALID_PRESETS.includes(state.themePreset)) {
            state.themePreset = 'paper-ledger';
          }
          document.body.setAttribute('data-theme', state.themePreset);
        }
      },
    }
  )
);
