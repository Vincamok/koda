import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    './src/**/*.{ts,tsx}',
    '../../packages/themes/src/**/*.{ts,tsx}',
    '../../packages/i18n/src/**/*.{ts,tsx}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        koda: {
          primary: '#6366f1',
          surface: '#1e1e2e',
          'surface-raised': '#2a2a3e',
          border: '#3b3b5c',
          text: '#e2e2f0',
          'text-muted': '#8888aa',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      borderRadius: {
        DEFAULT: '0.5rem',
      },
    },
  },
  plugins: [],
}

export default config
