/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
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
        status: {
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
        ink: {
          DEFAULT: 'hsl(var(--ink) / <alpha-value>)',
          soft: 'hsl(var(--ink-soft) / <alpha-value>)',
          faint: 'hsl(var(--ink-faint) / <alpha-value>)',
        }
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
    },
  },
  plugins: [],
}
