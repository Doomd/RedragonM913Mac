<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import {
		ACTION_GROUPS,
		ACTION_OPTIONS,
		SIDE_CALLOUTS,
		TOP_CALLOUTS,
		actionFromKeyboardEvent,
		fullActionLabel,
		shortActionLabel,
		type ButtonState
	} from '$lib/actions';

	let buttons = $state<ButtonState[]>([]);
	let draft = $state<Record<string, string>>({});
	let selectedId = $state<string>('side11');
	let busy = $state(false);
	let listeningForKey = $state(false);
	let statusMsg = $state('Ready — click "Read from Mouse" to sync.');
	let errorMsg = $state<string | null>(null);

	let buttonMap = $derived(new Map(buttons.map((b) => [b.id, b])));
	let selectedBtn = $derived(buttonMap.get(selectedId) ?? null);

	let pendingChanges = $derived.by(() => {
		const changes: Record<string, string> = {};
		for (const b of buttons) {
			const val = draft[b.id];
			if (val && val !== b.action) {
				changes[b.id] = val;
			}
		}
		return changes;
	});

	let pendingCount = $derived(Object.keys(pendingChanges).length);

	function currentAction(id: string): string {
		return draft[id] ?? buttonMap.get(id)?.action ?? '';
	}

	function isChangedFromFactory(id: string): boolean {
		const b = buttonMap.get(id);
		if (!b) return false;
		return currentAction(id) !== b.default_action;
	}

	function isPending(id: string): boolean {
		const b = buttonMap.get(id);
		if (!b) return false;
		return currentAction(id) !== b.action;
	}

	function syncDraftFromButtons(list: ButtonState[]) {
		buttons = list;
		const next: Record<string, string> = {};
		for (const b of list) {
			next[b.id] = b.action;
		}
		draft = next;
	}

	async function readFromMouse() {
		busy = true;
		errorMsg = null;
		statusMsg = 'Reading onboard memory from M913...';
		try {
			const list = await invoke<ButtonState[]>('read_mappings');
			syncDraftFromButtons(list);
			const modCount = list.filter((b) => b.is_modified).length;
			statusMsg = `Synced from mouse (${modCount} remapped from factory default).`;
		} catch (e) {
			errorMsg = String(e);
			statusMsg = 'Could not read from mouse.';
		} finally {
			busy = false;
		}
	}

	async function applyToMouse() {
		if (pendingCount === 0) return;
		busy = true;
		errorMsg = null;
		statusMsg = `Writing ${pendingCount} change(s) to mouse onboard memory...`;
		try {
			const list = await invoke<ButtonState[]>('apply_mappings', { updates: pendingChanges });
			syncDraftFromButtons(list);
			statusMsg = 'Saved and committed to mouse onboard memory.';
		} catch (e) {
			errorMsg = String(e);
			statusMsg = 'Failed to apply changes.';
		} finally {
			busy = false;
		}
	}

	async function resetAllToFactory() {
		busy = true;
		errorMsg = null;
		statusMsg = 'Resetting all 16 buttons to factory defaults...';
		try {
			const list = await invoke<ButtonState[]>('reset_all_defaults');
			syncDraftFromButtons(list);
			statusMsg = 'All buttons reset to factory defaults.';
		} catch (e) {
			errorMsg = String(e);
			statusMsg = 'Failed to reset mouse.';
		} finally {
			busy = false;
		}
	}

	function setSelectedAction(action: string) {
		if (!selectedBtn) return;
		draft = { ...draft, [selectedBtn.id]: action };
		listeningForKey = false;
	}

	function resetSelectedToDefault() {
		if (!selectedBtn) return;
		draft = { ...draft, [selectedBtn.id]: selectedBtn.default_action };
		listeningForKey = false;
	}

	function handleWindowKeydown(e: KeyboardEvent) {
		if (!listeningForKey) return;
		e.preventDefault();
		e.stopPropagation();
		const mapped = actionFromKeyboardEvent(e);
		if (mapped) {
			setSelectedAction(mapped);
		}
	}

	async function openGitHubRepo(e: MouseEvent) {
		e.preventDefault();
		try {
			await invoke('open_repo');
		} catch {
			window.open('https://github.com/Doomd/RedragonM913Mac', '_blank', 'noopener,noreferrer');
		}
	}

	onMount(async () => {
		try {
			const defaults = await invoke<ButtonState[]>('get_factory_defaults');
			syncDraftFromButtons(defaults);
		} catch {
			// Running in plain browser preview without Tauri runtime
		}
		await readFromMouse();
	});
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="min-h-screen flex flex-col bg-zinc-950 text-zinc-100">
	<!-- Top Toolbar -->
	<header class="border-b border-zinc-800 bg-zinc-900/70 px-5 py-3 flex items-center justify-between gap-4">
		<div class="flex items-center gap-3">
			<h1 class="text-base font-semibold tracking-tight">Redragon M913 Mapper</h1>
			<span class="text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-400">
				2.4GHz · Onboard Memory
			</span>
			<a
				href="https://github.com/Doomd/RedragonM913Mac"
				onclick={openGitHubRepo}
				title="View project on GitHub"
				class="inline-flex items-center gap-1.5 text-xs text-zinc-400 hover:text-zinc-100 transition"
			>
				<svg viewBox="0 0 16 16" class="w-3.5 h-3.5 fill-current" aria-hidden="true">
					<path
						d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
					/>
				</svg>
				<span>GitHub</span>
			</a>
		</div>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={readFromMouse}
				disabled={busy}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-zinc-700 disabled:opacity-50 transition cursor-pointer"
			>
				Read from Mouse
			</button>

			<button
				type="button"
				onclick={resetAllToFactory}
				disabled={busy}
				class="px-3 py-1.5 text-xs font-medium rounded-md bg-zinc-800 hover:bg-red-950 hover:text-red-200 border border-transparent hover:border-red-800/60 disabled:opacity-50 transition cursor-pointer"
			>
				Factory Reset All
			</button>

			<button
				type="button"
				onclick={applyToMouse}
				disabled={busy || pendingCount === 0}
				class="px-3.5 py-1.5 text-xs font-semibold rounded-md transition cursor-pointer {pendingCount > 0
					? 'bg-emerald-500 hover:bg-emerald-400 text-zinc-950'
					: 'bg-zinc-800 text-zinc-500 cursor-not-allowed'}"
			>
				{pendingCount > 0 ? `Apply to Mouse (${pendingCount})` : 'Saved'}
			</button>
		</div>
	</header>

	<!-- Status Banner -->
	<div class="px-5 py-2 text-xs border-b border-zinc-900 flex items-center justify-between gap-4 bg-zinc-950">
		<div class="flex items-center gap-2 min-w-0">
			{#if errorMsg}
				<span class="inline-block w-2 h-2 rounded-full bg-red-500 shrink-0"></span>
				<span class="text-red-400 truncate" title={errorMsg}>{errorMsg}</span>
			{:else}
				<span class="inline-block w-2 h-2 rounded-full {busy ? 'bg-amber-400 animate-pulse' : 'bg-emerald-500'} shrink-0"></span>
				<span class="text-zinc-400 truncate">{statusMsg}</span>
			{/if}
		</div>

		<div class="flex items-center gap-4 shrink-0 text-zinc-400">
			<span class="flex items-center gap-1.5">
				<span class="inline-block w-2.5 h-2.5 rounded-full bg-white border border-zinc-500"></span>
				Default
			</span>
			<span class="flex items-center gap-1.5">
				<span class="inline-block w-2.5 h-2.5 rounded-full bg-emerald-500"></span>
				Remapped
			</span>
		</div>
	</div>

	<!-- Main Workspace -->
	<main class="flex-1 grid grid-cols-1 lg:grid-cols-[1fr_300px] gap-4 p-5 items-start">
		<!-- Mouse Views with Interactive SVG Circle Callouts -->
		<div class="grid grid-cols-1 md:grid-cols-[1.55fr_1fr] gap-4">
			<!-- Side View (12 Buttons) -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4 flex flex-col">
				<div class="flex items-center justify-between mb-2">
					<h2 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
						Side View (12 Buttons)
					</h2>
					<span class="text-[11px] text-zinc-500">Click any circle to remap</span>
				</div>

				<div class="relative w-full my-auto rounded-lg bg-zinc-950/60 p-2">
					<img
						src="/images/side.webp"
						alt="Redragon M913 side view"
						class="w-full h-auto block select-none pointer-events-none"
					/>
					<svg
						viewBox="0 0 1346 600"
						class="absolute inset-0 w-full h-full p-2"
						role="group"
						aria-label="Side button callouts"
					>
						{#each SIDE_CALLOUTS as pt (pt.id)}
							{@const changed = isChangedFromFactory(pt.id)}
							{@const pending = isPending(pt.id)}
							{@const selected = selectedId === pt.id}
							{@const action = currentAction(pt.id)}
							<g
								class="cursor-pointer group"
								role="button"
								tabindex="0"
								aria-label="Button {pt.id}: {fullActionLabel(action)}"
								onclick={() => {
									selectedId = pt.id;
									listeningForKey = false;
								}}
								onkeydown={(e) => {
									if (e.key === 'Enter' || e.key === ' ') {
										selectedId = pt.id;
										listeningForKey = false;
									}
								}}
							>
								<title>{pt.id}: {fullActionLabel(action)}</title>
								{#if selected}
									<circle
										cx={pt.x}
										cy={pt.y}
										r="37"
										fill="none"
										stroke="#38bdf8"
										stroke-width="6"
									/>
								{/if}
								<circle
									cx={pt.x}
									cy={pt.y}
									r="28"
									fill={changed ? '#22c55e' : '#ffffff'}
									fill-opacity="0.92"
									stroke={pending ? '#facc15' : '#09090b'}
									stroke-width={pending ? '5' : '3'}
								/>
								<text
									x={pt.x}
									y={pt.y + 1}
									text-anchor="middle"
									dominant-baseline="central"
									font-size="22"
									font-weight="700"
									fill={changed ? '#052e16' : '#09090b'}
									class="select-none pointer-events-none"
								>
									{shortActionLabel(action)}
								</text>
							</g>
						{/each}
					</svg>
				</div>
			</section>

			<!-- Top View (Left, Right, Middle, Fire) -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4 flex flex-col">
				<div class="flex items-center justify-between mb-2">
					<h2 class="text-xs font-semibold uppercase tracking-wider text-zinc-400">
						Top View (4 Buttons)
					</h2>
					<span class="text-[11px] text-zinc-500">DPI buttons are fixed</span>
				</div>

				<div class="relative w-full max-w-[340px] mx-auto my-auto rounded-lg bg-zinc-950/60 p-2">
					<img
						src="/images/top.webp"
						alt="Redragon M913 top view"
						class="w-full h-auto block select-none pointer-events-none"
					/>
					<svg
						viewBox="0 0 900 1160"
						class="absolute inset-0 w-full h-full p-2"
						role="group"
						aria-label="Top button callouts"
					>
						{#each TOP_CALLOUTS as pt (pt.id)}
							{@const changed = isChangedFromFactory(pt.id)}
							{@const pending = isPending(pt.id)}
							{@const selected = selectedId === pt.id}
							{@const action = currentAction(pt.id)}
							<g
								class="cursor-pointer group"
								role="button"
								tabindex="0"
								aria-label="Button {pt.id}: {fullActionLabel(action)}"
								onclick={() => {
									selectedId = pt.id;
									listeningForKey = false;
								}}
								onkeydown={(e) => {
									if (e.key === 'Enter' || e.key === ' ') {
										selectedId = pt.id;
										listeningForKey = false;
									}
								}}
							>
								<title>{pt.id}: {fullActionLabel(action)}</title>
								{#if selected}
									<circle
										cx={pt.x}
										cy={pt.y}
										r="52"
										fill="none"
										stroke="#38bdf8"
										stroke-width="8"
									/>
								{/if}
								<circle
									cx={pt.x}
									cy={pt.y}
									r="40"
									fill={changed ? '#22c55e' : '#ffffff'}
									fill-opacity="0.92"
									stroke={pending ? '#facc15' : '#09090b'}
									stroke-width={pending ? '7' : '4'}
								/>
								<text
									x={pt.x}
									y={pt.y + 2}
									text-anchor="middle"
									dominant-baseline="central"
									font-size="30"
									font-weight="700"
									fill={changed ? '#052e16' : '#09090b'}
									class="select-none pointer-events-none"
								>
									{shortActionLabel(action)}
								</text>
							</g>
						{/each}
					</svg>
				</div>
			</section>
		</div>

		<!-- Right Inspector Panel -->
		<aside class="rounded-xl border border-zinc-800 bg-zinc-900/60 p-4 flex flex-col gap-4">
			{#if selectedBtn}
				{@const activeAction = currentAction(selectedBtn.id)}
				{@const changed = isChangedFromFactory(selectedBtn.id)}
				<div class="flex items-start justify-between gap-2">
					<div>
						<div class="text-xs uppercase tracking-wider text-zinc-400">Selected Button</div>
						<div class="text-lg font-semibold mt-0.5">
							{selectedBtn.id.startsWith('side') ? `Side ${selectedBtn.label}` : selectedBtn.label}
						</div>
					</div>
					<span
						class="text-[11px] font-medium px-2 py-0.5 rounded-full {changed
							? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40'
							: 'bg-zinc-800 text-zinc-300'}"
					>
						{changed ? 'Remapped' : 'Default'}
					</span>
				</div>

				<div class="rounded-lg bg-zinc-950 border border-zinc-800 p-3 text-xs space-y-1">
					<div class="flex justify-between">
						<span class="text-zinc-500">Current:</span>
						<span class="font-medium text-zinc-100">{fullActionLabel(activeAction)}</span>
					</div>
					<div class="flex justify-between">
						<span class="text-zinc-500">Factory Default:</span>
						<span class="text-zinc-400">{fullActionLabel(selectedBtn.default_action)}</span>
					</div>
				</div>

				<!-- Press any key capture -->
				<div>
					<button
						type="button"
						onclick={() => (listeningForKey = !listeningForKey)}
						class="w-full py-2 px-3 rounded-lg text-xs font-medium border transition cursor-pointer {listeningForKey
							? 'bg-sky-500/20 border-sky-400 text-sky-200 animate-pulse'
							: 'bg-zinc-800/80 hover:bg-zinc-800 border-zinc-700 text-zinc-200'}"
					>
						{listeningForKey ? 'Press any key or modifier now...' : 'Capture Key Press'}
					</button>
				</div>

				<!-- Quick macOS System Keys -->
				<div>
					<div class="text-[11px] font-semibold uppercase tracking-wider text-zinc-400 mb-2">
						Quick macOS System Keys
					</div>
					<div class="grid grid-cols-2 gap-1.5">
						{#each [
							{ v: 'option', t: '⌥ Option' },
							{ v: 'control', t: '⌃ Control' },
							{ v: 'command', t: '⌘ Command' },
							{ v: 'shift', t: '⇧ Shift' }
						] as mod (mod.v)}
							<button
								type="button"
								onclick={() => setSelectedAction(mod.v)}
								class="py-1.5 px-2.5 rounded-md text-xs font-medium border transition cursor-pointer {activeAction === mod.v
									? 'bg-emerald-500/20 border-emerald-500 text-emerald-200'
									: 'bg-zinc-950 hover:bg-zinc-800 border-zinc-800 text-zinc-300'}"
							>
								{mod.t}
							</button>
						{/each}
					</div>
				</div>

				<!-- Full Action Dropdown -->
				<div>
					<label
						for="action-select"
						class="block text-[11px] font-semibold uppercase tracking-wider text-zinc-400 mb-1.5"
					>
						All Keys & Actions
					</label>
					<select
						id="action-select"
						value={activeAction}
						onchange={(e) => setSelectedAction((e.currentTarget as HTMLSelectElement).value)}
						class="w-full rounded-lg bg-zinc-950 border border-zinc-700 px-2.5 py-2 text-xs text-zinc-100 focus:outline-none focus:border-emerald-500"
					>
						{#each ACTION_GROUPS as group (group)}
							<optgroup label={group}>
								{#each ACTION_OPTIONS.filter((o) => o.group === group) as opt (opt.value)}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</optgroup>
						{/each}
					</select>
				</div>

				<!-- Reset single button -->
				<button
					type="button"
					onclick={resetSelectedToDefault}
					disabled={!changed}
					class="w-full py-2 px-3 rounded-lg text-xs font-medium border border-zinc-700 bg-zinc-800/60 hover:bg-zinc-800 disabled:opacity-40 disabled:cursor-not-allowed transition cursor-pointer"
				>
					Reset Button to Default ({fullActionLabel(selectedBtn.default_action)})
				</button>
			{/if}
		</aside>
	</main>
</div>
