import type { Config } from 'tailwindcss'

export default {
	content: ['./index.html', './src/**/*.{vue,ts,tsx}', './stories/**/*.{ts,vue}', './.storybook/**/*.{ts,js}'],
	theme: {
		extend: {
			fontFamily: {
				sans: [
					'Inter',
					'SF Pro Display',
					'ui-sans-serif',
					'system-ui',
					'-apple-system',
					'BlinkMacSystemFont',
					'Segoe UI',
					'sans-serif'
				]
			},
			colors: {
				'h-bg': 'var(--h-color-bg)',
				'h-bg-muted': 'var(--h-color-bg-muted)',
				'h-surface': 'var(--h-color-surface)',
				'h-surface-raised': 'var(--h-color-surface-raised)',
				'h-text': 'var(--h-color-text)',
				'h-text-strong': 'var(--h-color-text-strong)',
				'h-text-muted': 'var(--h-color-text-muted)',
				'h-accent': 'var(--h-color-accent)',
				'h-danger': 'var(--h-color-danger)',
				'hh-bg': '#02090b',
				'hh-bg-raised': '#020d10',
				'hh-surface': '#06181b',
				'hh-surface-deep': '#041215',
				'hh-text': '#eefefb',
				'hh-text-strong': '#f2fffd',
				'hh-text-bright': '#ffffff',
				'hh-text-soft': '#dcefed',
				'hh-text-muted': '#91a8a8',
				'hh-text-subtle': '#8ea4a6',
				'hh-text-dim': '#849ca0',
				'hh-accent': '#2df0ce',
				'hh-accent-strong': '#25d8bd',
				'hh-accent-soft': '#9ee8df',
				'hh-accent-contrast': '#032522',
				'hh-danger': '#ffabab',
				'hh-danger-strong': '#ef3140',
				'hh-border-accent-soft': 'rgba(45, 240, 206, 0.18)',
				'hh-border-accent': 'rgba(45, 240, 206, 0.42)',
				'hh-border-subtle': 'rgba(111, 205, 195, 0.14)',
				'hh-border-muted': 'rgba(102, 189, 180, 0.1)',
				'hh-focus-ring': 'rgba(45, 240, 206, 0.62)',
				'hh-surface-tint': 'rgba(5, 22, 25, 0.78)',
				'hh-surface-panel': 'rgba(8, 29, 33, 0.94)',
				'hh-accent-tint': 'rgba(45, 240, 206, 0.08)',
				'hh-accent-control': 'rgba(25, 154, 132, 0.2)',
				'hh-danger-tint': 'rgba(128, 32, 40, 0.26)',
				'hh-status-accent-surface': 'rgba(45, 240, 206, 0.08)',
				'hh-status-accent-text': '#2df0ce',
				'hh-status-warning-surface': 'rgba(240, 170, 70, 0.16)',
				'hh-status-warning-text': '#f4c889',
				'hh-status-info-surface': 'rgba(120, 156, 240, 0.18)',
				'hh-status-info-text': '#aec6f7',
				'hh-status-success-surface': 'rgba(45, 214, 150, 0.16)',
				'hh-status-success-text': '#7fe6b4',
				'hh-status-danger-surface': 'rgba(128, 32, 40, 0.26)',
				'hh-status-danger-text': '#ffabab',
				'hh-status-archive-surface': 'rgba(176, 132, 240, 0.18)',
				'hh-status-archive-text': '#cdb2f2',
				'hh-status-neutral-surface': 'rgba(124, 156, 156, 0.12)',
				'hh-status-neutral-text': '#91a8a8'
			},
			spacing: {
				'hh-1': '4px',
				'hh-2': '8px',
				'hh-3': '12px',
				'hh-4': '16px',
				'hh-5': '20px',
				'hh-6': '24px'
			},
			borderRadius: {
				'hh-xs': '4px',
				'hh-sm': '6px',
				'hh-control': '7px',
				'hh-md': '8px',
				'hh-lg': '14px',
				'hh-xl': '18px',
				'hh-pill': '999px',
				'hh-round': '50%'
			},
			boxShadow: {
				'hh-sidebar':
					'inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28)',
				'hh-panel': 'inset 0 1px 0 rgba(255, 255, 255, 0.035)',
				'hh-modal': '0 24px 80px rgba(0, 0, 0, 0.55)'
			},
			minWidth: {
				'hh-shell': '800px',
				'hh-shell-content': '0px',
				'hh-shell-content-compact': '0px'
			},
			minHeight: {
				'hh-shell': '600px'
			},
			width: {
				'hh-sidebar': '224px',
				'hh-sidebar-compact': '208px',
				'hh-sidebar-rail': '64px'
			}
		}
	},
	plugins: []
} satisfies Config
