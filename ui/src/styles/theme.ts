export type ThemePreset = 'paper-ledger' | 'harbor-blueprint' | 'sage-archive';

export interface ThemePresetMetadata {
  id: ThemePreset;
  label: string;
}

export const THEME_PRESETS: ThemePresetMetadata[] = [
  { id: 'paper-ledger', label: '纸账本' },
  { id: 'harbor-blueprint', label: '港湾蓝图' },
  { id: 'sage-archive', label: '鼠尾档案' },
];

export const theme = {
  presets: {
    'paper-ledger': {
      canvas: '#F6F1E8',
      surface: '#FFFDF8',
      surfaceSubtle: '#F0E8DC',
      border: '#DDD2C2',
      ink: '#1F2A37',
      inkSoft: '#5E6A75',
      inkFaint: '#8C97A3',
      primary: '#1F6B75',
      success: '#2E8B57',
      warning: '#C77B18',
      error: '#C3513A',
      done: '#6C7A89',
      drifted: '#5F78B9',
      // Structural
      radius: '0.75rem',
      fontHeading: "'Noto Sans SC', sans-serif",
      borderStyle: 'solid',
    },
    'harbor-blueprint': {
      canvas: '#EEF3F6',
      surface: '#FCFEFF',
      surfaceSubtle: '#E2EBF0',
      border: '#C8D4DC',
      ink: '#1B2C3A',
      inkSoft: '#556776',
      inkFaint: '#7C8B98',
      primary: '#245A7A',
      success: '#2F7D5A',
      warning: '#B7791F',
      error: '#C0543F',
      done: '#667788',
      drifted: '#4F6FA8',
      // Structural
      radius: '0px',
      fontHeading: "'JetBrains Mono', monospace",
      borderStyle: 'solid',
    },
    'sage-archive': {
      canvas: '#F1F3EC',
      surface: '#FCFDF9',
      surfaceSubtle: '#E6EBDD',
      border: '#CFD7C5',
      ink: '#243128',
      inkSoft: '#5D6A60',
      inkFaint: '#879187',
      primary: '#4C6B4E',
      success: '#3E7C59',
      warning: '#B9852A',
      error: '#B85A46',
      done: '#6F7A70',
      drifted: '#6076A6',
      // Structural
      radius: '0.25rem',
      fontHeading: "'Noto Serif SC', serif",
      borderStyle: 'solid',
    }
  }
};
