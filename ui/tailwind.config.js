/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        /* ─── New design tokens ─── */
        canvas: 'var(--sl-canvas)',
        panel: 'var(--sl-panel)',
        surface: 'var(--sl-surface)',
        'border-subtle': 'var(--sl-border-subtle)',
        ink: {
          DEFAULT: 'var(--sl-ink)',
          secondary: 'var(--sl-ink-secondary)',
          muted: 'var(--sl-ink-muted)',
          soft: 'hsl(var(--ink-soft) / <alpha-value>)',
          faint: 'hsl(var(--ink-faint) / <alpha-value>)',
        },
        status: {
          active: 'hsl(var(--success) / <alpha-value>)',
          success: 'var(--sl-success)',
          'success-light': 'var(--sl-success-light)',
          warning: 'hsl(var(--warning) / <alpha-value>)',
          'warning-light': 'var(--sl-warning-light)',
          error: 'hsl(var(--error) / <alpha-value>)',
          'error-light': 'var(--sl-error-light)',
          done: 'hsl(var(--done) / <alpha-value>)',
          'done-light': 'var(--sl-done-light)',
          review: 'var(--sl-review)',
          'review-light': 'var(--sl-review-light)',
          drifted: 'hsl(var(--drifted) / <alpha-value>)',
          drift: 'var(--sl-drift)',
          'drift-light': 'var(--sl-drift-light)',
        },

        /* ─── Backward-compatible tokens (used by 580+ existing references) ─── */
        background: 'hsl(var(--canvas) / <alpha-value>)',
        foreground: 'hsl(var(--ink) / <alpha-value>)',
        card: {
          DEFAULT: 'hsl(var(--surface) / <alpha-value>)',
          foreground: 'hsl(var(--ink) / <alpha-value>)',
        },
        primary: {
          DEFAULT: 'hsl(var(--primary) / <alpha-value>)',
          hover: 'hsl(var(--primary-hover) / <alpha-value>)',
          foreground: 'hsl(var(--surface) / <alpha-value>)',
          light: 'var(--sl-primary-light)',
        },
        secondary: {
          DEFAULT: 'hsl(var(--surface-subtle) / <alpha-value>)',
          foreground: 'hsl(var(--ink) / <alpha-value>)',
        },
        muted: {
          DEFAULT: 'hsl(var(--surface-subtle) / <alpha-value>)',
          foreground: 'hsl(var(--ink-soft) / <alpha-value>)',
        },
        accent: {
          DEFAULT: 'hsl(var(--primary-tint) / <alpha-value>)',
          foreground: 'hsl(var(--ink) / <alpha-value>)',
        },
        destructive: {
          DEFAULT: 'hsl(var(--error) / <alpha-value>)',
          foreground: 'hsl(var(--surface) / <alpha-value>)',
        },
        border: 'hsl(var(--border) / <alpha-value>)',
        input: 'hsl(var(--border) / <alpha-value>)',
        ring: 'hsl(var(--focus-ring) / <alpha-value>)',
        'status-compat': {
          active: 'hsl(var(--success) / <alpha-value>)',
          warning: 'hsl(var(--warning) / <alpha-value>)',
          error: 'hsl(var(--error) / <alpha-value>)',
          done: 'hsl(var(--done) / <alpha-value>)',
          drifted: 'hsl(var(--drifted) / <alpha-value>)',
        },
        text: {
          primary: 'hsl(var(--ink) / <alpha-value>)',
          secondary: 'hsl(var(--ink-soft) / <alpha-value>)',
          muted: 'hsl(var(--ink-faint) / <alpha-value>)',
        },
        'ink-compat': {
          DEFAULT: 'hsl(var(--ink) / <alpha-value>)',
          soft: 'hsl(var(--ink-soft) / <alpha-value>)',
          faint: 'hsl(var(--ink-faint) / <alpha-value>)',
        },
      },
      fontFamily: {
        sans: ['Inter', 'PingFang SC', 'Noto Sans SC', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'SFMono-Regular', 'Cascadia Mono', 'ui-monospace', 'monospace'],
      },
      fontSize: {
        'metric': ['28px', { lineHeight: '32px', fontWeight: '600' }],
        'dense': ['13px', { lineHeight: '20px' }],
        'caption': ['12px', { lineHeight: '18px' }],
        'chip': ['11px', { lineHeight: '16px', fontWeight: '500' }],
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
      boxShadow: {
        'card': 'var(--sl-shadow-sm)',
        'elevated': 'var(--sl-shadow)',
        'overlay': 'var(--sl-shadow-lg)',
      },
      spacing: {
        'nav-collapsed': '48px',
        'nav-expanded': '180px',
        'list-collapsed': '280px',
      },
      transitionDuration: {
        'layout': '200ms',
      },
    },
  },
  plugins: [],
}
