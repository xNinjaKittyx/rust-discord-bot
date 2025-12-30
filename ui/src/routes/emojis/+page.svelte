<script lang="ts">
	import { onMount } from 'svelte';

	interface Emoji {
		id: string;
		name: string;
		animated: boolean;
		url: string;
		guild_id: string;
		guild_name: string;
	}

	interface EmojisResponse {
		emojis: Emoji[];
		count: number;
	}

	let emojis = $state<Emoji[]>([]);
	let filteredEmojis = $state<Emoji[]>([]);
	let searchText = $state('');
	let selectedGuild = $state('All');
	let guilds = $state<string[]>(['All']);
	let animatedFilter = $state<'all' | 'animated' | 'static'>('all');
	let sortBy = $state<'name-asc' | 'name-desc'>('name-asc');

	// Track previous filter values to avoid unnecessary updates
	let prevFilters = {
		searchText: '',
		selectedGuild: 'All',
		animatedFilter: 'all' as 'all' | 'animated' | 'static',
		sortBy: 'name-asc' as 'name-asc' | 'name-desc',
		emojisLength: 0
	};

	function updateFilteredEmojis() {
		let filtered = emojis;

		// Filter by guild
		if (selectedGuild !== 'All') {
			filtered = filtered.filter((emoji) => emoji.guild_name === selectedGuild);
		}

		// Filter by animated
		if (animatedFilter === 'animated') {
			filtered = filtered.filter((emoji) => emoji.animated);
		} else if (animatedFilter === 'static') {
			filtered = filtered.filter((emoji) => !emoji.animated);
		}

		// Filter by search text
		if (searchText.trim()) {
			const search = searchText.toLowerCase();
			filtered = filtered.filter((emoji) => emoji.name.toLowerCase().includes(search));
		}

		// Sort
		if (sortBy === 'name-asc') {
			filtered = [...filtered].sort((a, b) => a.name.localeCompare(b.name));
		} else if (sortBy === 'name-desc') {
			filtered = [...filtered].sort((a, b) => b.name.localeCompare(a.name));
		}

		filteredEmojis = filtered;
		
		// Update previous values
		prevFilters = {
			searchText,
			selectedGuild,
			animatedFilter,
			sortBy,
			emojisLength: emojis.length
		};
	}

	$effect(() => {
		// Only update if something actually changed
		if (
			searchText !== prevFilters.searchText ||
			selectedGuild !== prevFilters.selectedGuild ||
			animatedFilter !== prevFilters.animatedFilter ||
			sortBy !== prevFilters.sortBy ||
			emojis.length !== prevFilters.emojisLength
		) {
			updateFilteredEmojis();
		}
	});

	async function fetchEmojis() {
		try {
			const response = await fetch('/api/emojis');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: EmojisResponse = await response.json();
			emojis = data.emojis;

			// Extract unique guilds
			const uniqueGuilds = [...new Set(data.emojis.map((emoji) => emoji.guild_name))];
			guilds = ['All', ...uniqueGuilds.sort()];
		} catch (e) {
			console.error('Error fetching emojis:', e);
		}
	}

	function copyToClipboard(emoji: Emoji) {
		navigator.clipboard.writeText(emoji.id);
	}

	onMount(() => {
		fetchEmojis();
	});
</script>

<div class="max-w-7xl mx-auto">
	<h1 class="text-3xl font-bold text-white mb-6">Emojis</h1>

	<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 mb-6">
		<div class="flex flex-col md:flex-row gap-4 mb-4">
			<input
				type="text"
				bind:value={searchText}
				placeholder="Search emojis..."
				class="flex-1 px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500"
			/>
			<select
				bind:value={selectedGuild}
				class="px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white"
			>
				{#each guilds as guild (guild)}
					<option value={guild}>{guild}</option>
				{/each}
			</select>
			<select
				bind:value={animatedFilter}
				class="px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white"
			>
				<option value="all">All Types</option>
				<option value="animated">Animated Only</option>
				<option value="static">Static Only</option>
			</select>
			<select
				bind:value={sortBy}
				class="px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white"
			>
				<option value="name-asc">Name (A-Z)</option>
				<option value="name-desc">Name (Z-A)</option>
			</select>
		</div>
		<div class="mt-4 text-sm text-gray-400">
			Showing {filteredEmojis.length} of {emojis.length} emojis
		</div>
	</div>
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 gap-3">
        {#each filteredEmojis as emoji (emoji.id)}
            <button
                onclick={() => copyToClipboard(emoji)}
                class="bg-white/5 border border-white/10 rounded-lg p-3 hover:bg-white/10 transition-all hover:border-white/30 hover:scale-105 group relative"
                title="Click to copy"
            >
                <div class="flex flex-col items-center gap-2">
                    <img
                        src={emoji.url}
                        alt={emoji.name}
                        class="w-12 h-12 object-contain"
                        loading="lazy"
                        decoding="async"
                    />
                    <span class="text-xs text-gray-300 text-center truncate w-full group-hover:text-white">
                        :{emoji.name}:
                    </span>
                    {#if emoji.animated}
                        <span
                            class="absolute top-1 right-1 px-1.5 py-0.5 text-[10px] font-semibold bg-purple-500/30 text-purple-300 border border-purple-500/50 rounded"
                        >
                            GIF
                        </span>
                    {/if}
                </div>
                <div
                    class="absolute inset-0 bg-green-500/0 group-active:bg-green-500/20 rounded-lg transition-colors pointer-events-none"
                ></div>
            </button>
        {/each}
    </div>
</div>
