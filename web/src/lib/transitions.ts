// Svelte transitions that skip the animation when the user prefers reduced motion.
// Svelte animates with the Web Animations API, which CSS media queries can't reach.
import { prefersReducedMotion } from 'svelte/motion';
import {
	fade as svelteFade,
	fly as svelteFly,
	slide as svelteSlide,
	type FadeParams,
	type FlyParams,
	type SlideParams
} from 'svelte/transition';

function reduce<T extends { duration?: number }>(params: T | undefined): T | undefined {
	return prefersReducedMotion.current ? ({ ...params, duration: 0 } as T) : params;
}

export const fade = (node: Element, params?: FadeParams) => svelteFade(node, reduce(params));
export const fly = (node: Element, params?: FlyParams) => svelteFly(node, reduce(params));
export const slide = (node: Element, params?: SlideParams) => svelteSlide(node, reduce(params));
