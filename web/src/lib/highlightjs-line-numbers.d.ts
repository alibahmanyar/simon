import type { HLJSApi } from 'highlight.js';

export interface LineNumbersOptions {
	singleLine?: boolean;
	startFrom?: number;
}

export interface HLJSApiWithLineNumbers extends HLJSApi {
	initLineNumbersOnLoad(options?: LineNumbersOptions): void;
	lineNumbersBlock(block: HTMLElement, options?: LineNumbersOptions): void;
	lineNumbersBlockSync(block: HTMLElement, options?: LineNumbersOptions): void;
	lineNumbersValue(value: string, options?: LineNumbersOptions): string;
}

declare module 'highlightjs-line-numbers.js' {
	const plugin: void;
	export default plugin;
}
