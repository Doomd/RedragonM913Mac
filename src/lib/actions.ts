export interface ButtonState {
	id: string;
	label: string;
	action: string;
	default_action: string;
	is_modified: boolean;
}

export interface ActionOption {
	value: string;
	label: string;
	short: string;
	group: 'macOS Modifiers' | 'Mouse & Special' | 'Numbers & Symbols' | 'Letters' | 'Function Keys' | 'Navigation';
}

export const ACTION_OPTIONS: ActionOption[] = [
	// macOS Modifiers
	{ value: 'option', label: 'Option (⌥ Left)', short: '⌥', group: 'macOS Modifiers' },
	{ value: 'control', label: 'Control (⌃ Left)', short: '⌃', group: 'macOS Modifiers' },
	{ value: 'command', label: 'Command (⌘ Left)', short: '⌘', group: 'macOS Modifiers' },
	{ value: 'shift', label: 'Shift (⇧ Left)', short: '⇧', group: 'macOS Modifiers' },
	{ value: 'option_r', label: 'Option (⌥ Right)', short: '⌥R', group: 'macOS Modifiers' },
	{ value: 'control_r', label: 'Control (⌃ Right)', short: '⌃R', group: 'macOS Modifiers' },
	{ value: 'command_r', label: 'Command (⌘ Right)', short: '⌘R', group: 'macOS Modifiers' },
	{ value: 'shift_r', label: 'Shift (⇧ Right)', short: '⇧R', group: 'macOS Modifiers' },

	// Mouse & Special
	{ value: 'left', label: 'Left Click', short: 'L', group: 'Mouse & Special' },
	{ value: 'right', label: 'Right Click', short: 'R', group: 'Mouse & Special' },
	{ value: 'middle', label: 'Middle Click', short: 'M', group: 'Mouse & Special' },
	{ value: 'backward', label: 'Mouse Back', short: 'Bk', group: 'Mouse & Special' },
	{ value: 'forward', label: 'Mouse Forward', short: 'Fw', group: 'Mouse & Special' },
	{ value: 'fire', label: 'Fire (Triple Click)', short: '3x', group: 'Mouse & Special' },
	{ value: 'led_toggle', label: 'Toggle LED', short: 'LED', group: 'Mouse & Special' },
	{ value: 'none', label: 'Disabled (None)', short: '∅', group: 'Mouse & Special' },

	// Numbers & Symbols
	...['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'].map((n) => ({
		value: n,
		label: `Key ${n}`,
		short: n,
		group: 'Numbers & Symbols' as const
	})),
	{ value: 'numplus', label: 'Numpad + (Factory 11)', short: '+', group: 'Numbers & Symbols' },
	{ value: 'numminus', label: 'Numpad - (Factory 12)', short: '-', group: 'Numbers & Symbols' },
	{ value: 'minus', label: 'Minus (-)', short: '-', group: 'Numbers & Symbols' },
	{ value: 'equal', label: 'Equal (=)', short: '=', group: 'Numbers & Symbols' },
	{ value: 'lbracket', label: 'Left Bracket ([)', short: '[', group: 'Numbers & Symbols' },
	{ value: 'rbracket', label: 'Right Bracket (])', short: ']', group: 'Numbers & Symbols' },
	{ value: 'backslash', label: 'Backslash (\\)', short: '\\', group: 'Numbers & Symbols' },
	{ value: 'semicolon', label: 'Semicolon (;)', short: ';', group: 'Numbers & Symbols' },
	{ value: 'quote', label: "Quote (')", short: "'", group: 'Numbers & Symbols' },
	{ value: 'grave', label: 'Grave (`)', short: '`', group: 'Numbers & Symbols' },
	{ value: 'comma', label: 'Comma (,)', short: ',', group: 'Numbers & Symbols' },
	{ value: 'dot', label: 'Period (.)', short: '.', group: 'Numbers & Symbols' },
	{ value: 'slash', label: 'Slash (/)', short: '/', group: 'Numbers & Symbols' },

	// Letters
	...'abcdefghijklmnopqrstuvwxyz'.split('').map((ch) => ({
		value: ch,
		label: `Key ${ch.toUpperCase()}`,
		short: ch.toUpperCase(),
		group: 'Letters' as const
	})),

	// Function Keys
	...Array.from({ length: 20 }, (_, i) => ({
		value: `f${i + 1}`,
		label: `F${i + 1}`,
		short: `F${i + 1}`,
		group: 'Function Keys' as const
	})),

	// Navigation
	{ value: 'enter', label: 'Return / Enter', short: '↵', group: 'Navigation' },
	{ value: 'esc', label: 'Escape', short: 'Esc', group: 'Navigation' },
	{ value: 'tab', label: 'Tab', short: '⇥', group: 'Navigation' },
	{ value: 'space', label: 'Space', short: '␣', group: 'Navigation' },
	{ value: 'backspace', label: 'Backspace (⌫)', short: '⌫', group: 'Navigation' },
	{ value: 'delete', label: 'Forward Delete (⌦)', short: '⌦', group: 'Navigation' },
	{ value: 'arrow_up', label: 'Arrow Up', short: '↑', group: 'Navigation' },
	{ value: 'arrow_down', label: 'Arrow Down', short: '↓', group: 'Navigation' },
	{ value: 'arrow_left', label: 'Arrow Left', short: '←', group: 'Navigation' },
	{ value: 'arrow_right', label: 'Arrow Right', short: '→', group: 'Navigation' },
	{ value: 'home', label: 'Home', short: 'Hm', group: 'Navigation' },
	{ value: 'end', label: 'End', short: 'End', group: 'Navigation' },
	{ value: 'pageup', label: 'Page Up', short: 'PgU', group: 'Navigation' },
	{ value: 'pagedown', label: 'Page Down', short: 'PgD', group: 'Navigation' }
];

export const ACTION_GROUPS = [
	'macOS Modifiers',
	'Mouse & Special',
	'Numbers & Symbols',
	'Letters',
	'Function Keys',
	'Navigation'
] as const;

const BY_VALUE = new Map(ACTION_OPTIONS.map((o) => [o.value, o]));

export function shortActionLabel(action: string): string {
	return BY_VALUE.get(action)?.short ?? action.slice(0, 3).toUpperCase();
}

export function fullActionLabel(action: string): string {
	return BY_VALUE.get(action)?.label ?? action;
}

/** Map a browser KeyboardEvent to one of our canonical action values. */
export function actionFromKeyboardEvent(e: KeyboardEvent): string | null {
	if (e.code === 'AltLeft') return 'option';
	if (e.code === 'AltRight') return 'option_r';
	if (e.code === 'ControlLeft') return 'control';
	if (e.code === 'ControlRight') return 'control_r';
	if (e.code === 'MetaLeft') return 'command';
	if (e.code === 'MetaRight') return 'command_r';
	if (e.code === 'ShiftLeft') return 'shift';
	if (e.code === 'ShiftRight') return 'shift_r';

	if (e.code.startsWith('Key') && e.code.length === 4) {
		return e.code[3].toLowerCase();
	}
	if (e.code.startsWith('Digit') && e.code.length === 6) {
		return e.code[5];
	}
	if (/^F([1-9]|1[0-9]|20)$/.test(e.code)) {
		return e.code.toLowerCase();
	}

	const map: Record<string, string> = {
		Enter: 'enter',
		Escape: 'esc',
		Backspace: 'backspace',
		Tab: 'tab',
		Space: 'space',
		Minus: 'minus',
		Equal: 'equal',
		BracketLeft: 'lbracket',
		BracketRight: 'rbracket',
		Backslash: 'backslash',
		Semicolon: 'semicolon',
		Quote: 'quote',
		Backquote: 'grave',
		Comma: 'comma',
		Period: 'dot',
		Slash: 'slash',
		Delete: 'delete',
		Home: 'home',
		End: 'end',
		PageUp: 'pageup',
		PageDown: 'pagedown',
		ArrowUp: 'arrow_up',
		ArrowDown: 'arrow_down',
		ArrowLeft: 'arrow_left',
		ArrowRight: 'arrow_right',
		NumpadAdd: 'numplus',
		NumpadSubtract: 'numminus'
	};
	return map[e.code] ?? null;
}

export interface CalloutCoord {
	id: string;
	x: number;
	y: number;
}

// Coordinates match static/images/side.webp (1346 x 600)
export const SIDE_CALLOUTS: CalloutCoord[] = [
	{ id: 'side3', x: 535, y: 185 },
	{ id: 'side6', x: 630, y: 185 },
	{ id: 'side9', x: 735, y: 185 },
	{ id: 'side12', x: 830, y: 185 },
	{ id: 'side2', x: 535, y: 278 },
	{ id: 'side5', x: 630, y: 278 },
	{ id: 'side8', x: 735, y: 278 },
	{ id: 'side11', x: 830, y: 278 },
	{ id: 'side1', x: 535, y: 368 },
	{ id: 'side4', x: 630, y: 368 },
	{ id: 'side7', x: 735, y: 368 },
	{ id: 'side10', x: 830, y: 368 }
];

// Coordinates match static/images/top.webp (900 x 1160)
export const TOP_CALLOUTS: CalloutCoord[] = [
	{ id: 'fire', x: 207, y: 265 },
	{ id: 'left', x: 315, y: 280 },
	{ id: 'middle', x: 450, y: 265 },
	{ id: 'right', x: 600, y: 280 }
];
