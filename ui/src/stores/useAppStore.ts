import { create } from 'zustand';
import { ProjectUIState } from '../types';

interface AppState {
  projectUIStates: Record<string, ProjectUIState>;
  
  // Actions
  updateProjectUIState: (projectId: string, updates: Partial<ProjectUIState>) => void;
}

export const useAppStore = create<AppState>((set) => ({
  projectUIStates: {},

  updateProjectUIState: (projectId, updates) => set((state) => ({
    projectUIStates: {
      ...state.projectUIStates,
      [projectId]: {
        ...(state.projectUIStates[projectId] || { activeTab: 'inbox', selectedObjectId: null, filters: {} }),
        ...updates
      }
    }
  })),
}));
