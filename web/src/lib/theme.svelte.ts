import { browser } from '$app/environment';
import type { Chart } from 'chart.js';

export type Theme = 'light' | 'dark';

// Keep in sync with the inline script in app.html and static/auth.html
const STORAGE_KEY = 'simon-theme';
const LIGHT_QUERY = '(prefers-color-scheme: light)';

function storedTheme(): Theme | null {
	try {
		const t = localStorage.getItem(STORAGE_KEY);
		return t === 'light' || t === 'dark' ? t : null;
	} catch {
		return null;
	}
}

function systemTheme(): Theme {
	return matchMedia(LIGHT_QUERY).matches ? 'light' : 'dark';
}

export const theme = $state<{ current: Theme }>({
	current: browser ? (storedTheme() ?? systemTheme()) : 'dark'
});

function applyTheme(t: Theme) {
	theme.current = t;
	document.documentElement.dataset.theme = t;
}

export function toggleTheme() {
	const next = theme.current === 'dark' ? 'light' : 'dark';
	applyTheme(next);
	try {
		localStorage.setItem(STORAGE_KEY, next);
	} catch {
		// Storage unavailable; the choice just won't persist
	}
}

if (browser) {
	applyTheme(theme.current);
	// Follow the OS setting until the user picks a theme explicitly
	matchMedia(LIGHT_QUERY).addEventListener('change', (e) => {
		if (!storedTheme()) applyTheme(e.matches ? 'light' : 'dark');
	});
}

function cssVar(name: string) {
	return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

// Colors for canvas-based charts, which can't use CSS variables directly.
// Reading this inside an effect or $derived re-runs it when the theme changes.
export function getChartColors() {
	void theme.current;
	return {
		text: cssVar('--text'),
		textSoft: cssVar('--text-soft'),
		grid: `rgba(${cssVar('--fg-rgb')}, 0.1)`,
		gridSolid: cssVar('--chart-grid'),
		background: cssVar('--bg-elevated')
	};
}

export function applyChartJsColors(chart: Chart, colors = getChartColors()) {
	for (const scale of Object.values(chart.options.scales ?? {})) {
		if (!scale) continue;
		if (scale.grid) scale.grid.color = colors.grid;
		if (scale.ticks) scale.ticks.color = colors.text;
		if ('title' in scale && scale.title) scale.title.color = colors.text;
	}
	const legend = chart.options.plugins?.legend;
	if (legend?.labels) legend.labels.color = colors.text;
	chart.update();
}
