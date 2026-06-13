<script lang="ts">
	import './hermesSelect.css';
	import { tick } from 'svelte';
	import Icon from '@iconify/svelte';

	type SelectOption = {
		value: string;
		label: string;
		eyebrow?: string;
		description?: string;
		meta?: string;
		disabled?: boolean;
		disabledReason?: string;
	};

	type SelectGroup = {
		label: string;
		options: SelectOption[];
	};

	type Props = {
		value: string;
		options?: SelectOption[];
		groups?: SelectGroup[];
		placeholder?: string;
		searchPlaceholder?: string;
		emptyLabel?: string;
		ariaLabel?: string;
		disabled?: boolean;
		searchable?: boolean;
		compact?: boolean;
		onChange: (value: string) => void;
	};

	const MENU_MIN_HEIGHT = 88;
	const MENU_THEME_VARIABLES = [
		'--hh-color-text',
		'--hh-color-text-muted',
		'--hh-color-text-soft',
		'--hh-color-accent',
		'--hh-color-accent-soft',
		'--hh-color-accent-contrast',
		'--hh-border-accent',
		'--hh-border-accent-soft',
		'--hh-focus-ring',
		'--hh-accent-tint'
	];

	let {
		value,
		options = [],
		groups = [],
		placeholder = 'Select',
		searchPlaceholder = 'Search',
		emptyLabel = 'No options',
		ariaLabel = '',
		disabled = false,
		searchable = true,
		compact = false,
		onChange
	}: Props = $props();

	let rootElement: HTMLDivElement | null = $state(null);
	let triggerElement: HTMLButtonElement | null = $state(null);
	let menuElement: HTMLDivElement | null = $state(null);
	let searchElement: HTMLInputElement | null = $state(null);
	let isOpen = $state(false);
	let query = $state('');
	let activeValue = $state('');
	let menuPlacement = $state<'below' | 'above'>('below');

	const normalizedGroups = $derived(normalizeGroups(groups, options));
	const filteredGroups = $derived(filterGroups(normalizedGroups, query));
	const enabledOptions = $derived(
		filteredGroups.flatMap((group) => group.options).filter((option) => !option.disabled)
	);
	const selectedOption = $derived(findOption(normalizedGroups, value));
	const visibleOptionCount = $derived(
		filteredGroups.reduce((count, group) => count + group.options.length, 0)
	);

	$effect(() => {
		if (!isOpen || typeof document === 'undefined') return;

		const handlePointerDown = (event: PointerEvent) => {
			const target = event.target as Node;
			if (rootElement?.contains(target) || menuElement?.contains(target)) return;
			closeSelect();
		};
		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === 'Escape') {
				event.preventDefault();
				closeSelect();
				triggerElement?.focus();
			}
		};

		document.addEventListener('pointerdown', handlePointerDown);
		document.addEventListener('keydown', handleKeyDown);

		return () => {
			document.removeEventListener('pointerdown', handlePointerDown);
			document.removeEventListener('keydown', handleKeyDown);
		};
	});

	$effect(() => {
		if (!isOpen || typeof window === 'undefined') return;

		const update = () => updateMenuPosition();
		window.addEventListener('resize', update);
		window.addEventListener('scroll', update, true);

		return () => {
			window.removeEventListener('resize', update);
			window.removeEventListener('scroll', update, true);
		};
	});

	function normalizeGroups(groupedOptions: SelectGroup[], flatOptions: SelectOption[]): SelectGroup[] {
		if (groupedOptions.length > 0) {
			return groupedOptions.map((group) => ({
				label: group.label,
				options: group.options.map(normalizeOption)
			}));
		}
		return [{ label: '', options: flatOptions.map(normalizeOption) }];
	}

	function normalizeOption(option: SelectOption): SelectOption {
		return {
			...option,
			disabled: option.disabled || Boolean(option.disabledReason)
		};
	}

	function filterGroups(items: SelectGroup[], searchTerm: string): SelectGroup[] {
		const normalizedQuery = searchTerm.trim().toLowerCase();
		if (!normalizedQuery) {
			return items;
		}

		return items
			.map((group) => ({
				label: group.label,
				options: group.options.filter((option) => optionMatches(option, group.label, normalizedQuery))
			}))
			.filter((group) => group.options.length > 0);
	}

	function optionMatches(option: SelectOption, groupLabel: string, searchTerm: string): boolean {
		return [
			groupLabel,
			option.label,
			option.eyebrow ?? '',
			option.description ?? '',
			option.meta ?? '',
			option.disabledReason ?? ''
		]
			.join(' ')
			.toLowerCase()
			.includes(searchTerm);
	}

	function findOption(items: SelectGroup[], optionValue: string): SelectOption | null {
		for (const group of items) {
			const option = group.options.find((item) => item.value === optionValue);
			if (option) return option;
		}
		return null;
	}

	async function openSelect() {
		if (disabled) return;
		query = '';
		isOpen = true;
		activeValue = selectedOption && !selectedOption.disabled ? selectedOption.value : enabledOptions[0]?.value ?? '';
		await tick();
		updateMenuPosition();
		if (searchable) {
			searchElement?.focus({ preventScroll: true });
		}
		requestAnimationFrame(() => updateMenuPosition());
	}

	function closeSelect() {
		isOpen = false;
		query = '';
	}

	function toggleSelect() {
		if (isOpen) {
			closeSelect();
		} else {
			void openSelect();
		}
	}

	function updateMenuPosition() {
		if (typeof window === 'undefined' || !triggerElement || !menuElement) return;

		syncMenuTheme();
		const rect = triggerElement.getBoundingClientRect();
		const viewportMargin = 12;
		const gap = 6;
		const preferredMenuHeight = 360;
		const bounds = floatingBounds(triggerElement, viewportMargin);
		const availableWidth = Math.max(220, bounds.right - bounds.left);
		const maxMenuWidth = Math.min(560, availableWidth);
		const minMenuWidth = Math.min(260, maxMenuWidth);
		const width = Math.max(minMenuWidth, Math.min(rect.width, maxMenuWidth));
		const preferredLeft = rect.left + width > bounds.right ? rect.right - width : rect.left;
		const left = clamp(preferredLeft, bounds.left, bounds.right - width);
		const naturalHeight = Math.min(
			preferredMenuHeight,
			Math.max(MENU_MIN_HEIGHT, menuElement.scrollHeight)
		);
		const spaceBelow = bounds.bottom - rect.bottom - gap;
		const spaceAbove = rect.top - bounds.top - gap;
		const shouldOpenAbove =
			spaceBelow < Math.min(naturalHeight, 220) && spaceAbove > spaceBelow;
		const availableHeight = Math.max(
			MENU_MIN_HEIGHT,
			shouldOpenAbove ? spaceAbove : spaceBelow
		);
		const maxHeight = Math.min(naturalHeight, availableHeight);
		let top = shouldOpenAbove ? rect.top - gap - maxHeight : rect.bottom + gap;
		top = clamp(top, bounds.top, bounds.bottom - maxHeight);
		menuPlacement = shouldOpenAbove ? 'above' : 'below';

		menuElement.style.setProperty('--hermes-select-top', `${Math.round(top)}px`);
		menuElement.style.setProperty('--hermes-select-left', `${Math.round(left)}px`);
		menuElement.style.setProperty('--hermes-select-width', `${Math.round(width)}px`);
		menuElement.style.setProperty('--hermes-select-max-height', `${Math.round(maxHeight)}px`);
	}

	function syncMenuTheme() {
		if (!triggerElement || !menuElement) return;
		const computedStyles = window.getComputedStyle(triggerElement);
		for (const variableName of MENU_THEME_VARIABLES) {
			const value = computedStyles.getPropertyValue(variableName).trim();
			if (value) {
				menuElement.style.setProperty(variableName, value);
			}
		}
	}

	function floatingBounds(anchor: HTMLElement, margin: number) {
		let bounds = {
			top: margin,
			right: window.innerWidth - margin,
			bottom: window.innerHeight - margin,
			left: margin
		};
		let parent = anchor.parentElement;

		while (parent && parent !== document.body) {
			const computedStyles = window.getComputedStyle(parent);
			const clipsX = clipsOverflow(computedStyles.overflowX);
			const clipsY = clipsOverflow(computedStyles.overflowY);
			if (clipsX || clipsY) {
				const parentRect = parent.getBoundingClientRect();
				if (parentRect.width > 0 && parentRect.height > 0) {
					bounds = {
						top: clipsY ? Math.max(bounds.top, parentRect.top + margin) : bounds.top,
						right: clipsX ? Math.min(bounds.right, parentRect.right - margin) : bounds.right,
						bottom: clipsY ? Math.min(bounds.bottom, parentRect.bottom - margin) : bounds.bottom,
						left: clipsX ? Math.max(bounds.left, parentRect.left + margin) : bounds.left
					};
				}
			}
			parent = parent.parentElement;
		}

		if (bounds.right - bounds.left < 220 || bounds.bottom - bounds.top < MENU_MIN_HEIGHT) {
			return {
				top: margin,
				right: window.innerWidth - margin,
				bottom: window.innerHeight - margin,
				left: margin
			};
		}
		return bounds;
	}

	function clipsOverflow(value: string): boolean {
		return ['auto', 'scroll', 'hidden', 'clip'].includes(value);
	}

	function clamp(value: number, min: number, max: number): number {
		return Math.min(Math.max(value, min), max);
	}

	function portal(node: HTMLElement) {
		document.body.appendChild(node);
		return {
			destroy() {
				node.remove();
			}
		};
	}

	function selectOption(option: SelectOption) {
		if (option.disabled) return;
		onChange(option.value);
		closeSelect();
		triggerElement?.focus();
	}

	function handleTriggerKeydown(event: KeyboardEvent) {
		if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) {
			event.preventDefault();
			void openSelect();
		}
	}

	function handleMenuKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			moveActiveOption(1);
		}
		if (event.key === 'ArrowUp') {
			event.preventDefault();
			moveActiveOption(-1);
		}
		if (event.key === 'Enter' && activeValue) {
			event.preventDefault();
			const option = findOption(filteredGroups, activeValue);
			if (option) selectOption(option);
		}
	}

	function moveActiveOption(direction: number) {
		if (enabledOptions.length === 0) return;
		const currentIndex = enabledOptions.findIndex((option) => option.value === activeValue);
		const nextIndex =
			currentIndex === -1
				? 0
				: (currentIndex + direction + enabledOptions.length) % enabledOptions.length;
		activeValue = enabledOptions[nextIndex].value;
	}
</script>

<div class="hermes-select" class:open={isOpen} class:compact class:disabled bind:this={rootElement}>
	<button
		bind:this={triggerElement}
		type="button"
		class="hermes-select-trigger"
		aria-label={ariaLabel || placeholder}
		aria-haspopup="listbox"
		aria-expanded={isOpen}
		disabled={disabled}
		onclick={toggleSelect}
		onkeydown={handleTriggerKeydown}
	>
		<span class="hermes-select-value" class:placeholder={!selectedOption}>
			<strong>{selectedOption?.label ?? placeholder}</strong>
			{#if selectedOption?.meta}
				<small>{selectedOption.meta}</small>
			{:else if selectedOption?.description}
				<small>{selectedOption.description}</small>
			{/if}
		</span>
		<Icon icon="tabler:chevron-down" width="17" height="17" />
	</button>

	{#if isOpen}
		<div
			use:portal
			bind:this={menuElement}
			class="hermes-select-menu"
			class:above={menuPlacement === 'above'}
			class:below={menuPlacement === 'below'}
			role="presentation"
			onkeydown={handleMenuKeydown}
		>
			{#if searchable}
				<label class="hermes-select-search">
					<Icon icon="tabler:search" width="15" height="15" />
					<input
						bind:this={searchElement}
						value={query}
						placeholder={searchPlaceholder}
						aria-label={searchPlaceholder}
						oninput={(event) => {
							query = (event.currentTarget as HTMLInputElement).value;
							activeValue = enabledOptions[0]?.value ?? '';
						}}
					/>
				</label>
			{/if}
			<div class="hermes-select-list" role="listbox" aria-label={ariaLabel || placeholder}>
				{#if visibleOptionCount === 0}
					<div class="hermes-select-empty">{emptyLabel}</div>
				{:else}
					{#each filteredGroups as group}
						{#if group.label}
							<div class="hermes-select-group-label">{group.label}</div>
						{/if}
						{#each group.options as option}
							<button
								type="button"
								class="hermes-select-option"
								class:active={option.value === activeValue}
								class:selected={option.value === value}
								disabled={option.disabled}
								role="option"
								aria-selected={option.value === value}
								onclick={() => selectOption(option)}
								onmouseenter={() => {
									if (!option.disabled) activeValue = option.value;
								}}
							>
								<span>
									<strong>{option.label}</strong>
									{#if option.description}
										<small>{option.description}</small>
									{/if}
									{#if option.disabledReason}
										<em>{option.disabledReason}</em>
									{/if}
								</span>
								{#if option.eyebrow}
									<mark>{option.eyebrow}</mark>
								{/if}
								{#if option.value === value}
									<Icon icon="tabler:check" width="16" height="16" />
								{/if}
							</button>
						{/each}
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>
