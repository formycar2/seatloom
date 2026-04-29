/**
 * SeatLoom Design Tokens
 *
 * Single light theme. Colors defined as CSS custom properties in globals.css,
 * re-exported here as constants for programmatic use (charts, SVG, etc.)
 */

export const colors = {
  canvas: '#FFFFFF',
  panel: '#F8F9FA',
  surface: '#FFFFFF',
  border: '#E5E7EB',
  borderSubtle: '#F3F4F6',

  ink: '#1A1A2E',
  inkSecondary: '#6B7280',
  inkMuted: '#9CA3AF',

  primary: '#3B82F6',
  primaryHover: '#2563EB',
  primaryLight: '#EFF6FF',

  success: '#22C55E',
  successLight: '#F0FDF4',
  warning: '#F59E0B',
  warningLight: '#FFFBEB',
  error: '#EF4444',
  errorLight: '#FEF2F2',
  done: '#94A3B8',
  doneLight: '#F8FAFC',
  review: '#14B8A6',
  reviewLight: '#F0FDFA',
  drift: '#8B5CF6',
  driftLight: '#F5F3FF',
} as const;

/** Chart color series (4 colors) */
export const chartColors = [
  colors.primary,   // blue
  colors.drift,     // violet
  colors.review,    // teal
  colors.warning,   // amber
] as const;

/** Breakpoints in px */
export const breakpoints = {
  mobile: 768,
  tablet: 1024,
  compact: 1280,
} as const;

/** Nav rail widths */
export const navWidth = {
  collapsed: 48,
  expanded: 180,
} as const;

/** Master-detail list width when collapsed */
export const listCollapsedWidth = 280;
