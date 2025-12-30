<script lang="ts">
	import { onMount } from 'svelte';

	interface Stats {
		servers: number;
		commands_24h: number;
		tags: number;
		emojis: number;
	}

	let stats = $state<Stats | null>(null);
	let isLoading = $state(true);
	let error = $state('');

	async function fetchStats() {
		isLoading = true;
		error = '';
		try {
			const response = await fetch('/api/stats');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			stats = await response.json();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch stats';
			console.error('Error fetching stats:', e);
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		fetchStats();
		// Auto-refresh every 30 seconds
		const interval = setInterval(fetchStats, 30000);
		return () => clearInterval(interval);
	});
</script>

<div class="max-w-6xl mx-auto">
	<div class="mb-8">
		<h1 class="text-4xl font-bold text-white mb-2">Dashboard</h1>
		<p class="text-gray-400 text-lg">
			Welcome to the Discord bot management interface
		</p>
	</div>

	{#if error}
		<div class="bg-red-500/10 border border-red-500/50 text-red-400 rounded-lg p-4 mb-6">
			<strong>Error:</strong>
			{error}
		</div>
	{/if}

	{#if isLoading && !stats}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
			{#each Array(4) as _}
				<div
					class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 animate-pulse"
				>
					<div class="h-4 bg-white/10 rounded w-20 mb-4"></div>
					<div class="h-10 bg-white/10 rounded w-16"></div>
				</div>
			{/each}
		</div>
	{:else if stats}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
			<div
				class="bg-gradient-to-br from-blue-500/10 to-blue-600/10 backdrop-blur-sm rounded-lg shadow-lg border border-blue-500/20 p-6 hover:border-blue-500/40 transition-all"
			>
				<div class="flex items-center justify-between mb-2">
					<h3 class="text-sm font-medium text-blue-300 uppercase tracking-wide">Servers</h3>
					<svg class="w-8 h-8 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
						/>
					</svg>
				</div>
				<p class="text-4xl font-bold text-white">{stats.servers}</p>
			</div>

			<div
				class="bg-gradient-to-br from-green-500/10 to-green-600/10 backdrop-blur-sm rounded-lg shadow-lg border border-green-500/20 p-6 hover:border-green-500/40 transition-all"
			>
				<div class="flex items-center justify-between mb-2">
					<h3 class="text-sm font-medium text-green-300 uppercase tracking-wide">Commands 24h</h3>
					<svg class="w-8 h-8 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M13 10V3L4 14h7v7l9-11h-7z"
						/>
					</svg>
				</div>
				<p class="text-4xl font-bold text-white">{stats.commands_24h}</p>
			</div>

			<div
				class="bg-gradient-to-br from-purple-500/10 to-purple-600/10 backdrop-blur-sm rounded-lg shadow-lg border border-purple-500/20 p-6 hover:border-purple-500/40 transition-all"
			>
				<div class="flex items-center justify-between mb-2">
					<h3 class="text-sm font-medium text-purple-300 uppercase tracking-wide">Tags</h3>
					<svg class="w-8 h-8 text-purple-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"
						/>
					</svg>
				</div>
				<p class="text-4xl font-bold text-white">{stats.tags}</p>
			</div>

			<div
				class="bg-gradient-to-br from-yellow-500/10 to-yellow-600/10 backdrop-blur-sm rounded-lg shadow-lg border border-yellow-500/20 p-6 hover:border-yellow-500/40 transition-all"
			>
				<div class="flex items-center justify-between mb-2">
					<h3 class="text-sm font-medium text-yellow-300 uppercase tracking-wide">Emojis</h3>
					<svg class="w-8 h-8 text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</div>
				<p class="text-4xl font-bold text-white">{stats.emojis}</p>
			</div>
		</div>
	{/if}

	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<a
			href="/logging"
			class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 hover:border-white/20 hover:bg-white/10 transition-all group"
		>
			<h2 class="text-2xl font-semibold text-white mb-2 group-hover:text-blue-300 transition-colors">
				📋 Logging
			</h2>
			<p class="text-gray-400">View and filter bot logs in real-time</p>
		</a>

		<a
			href="/commands"
			class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 hover:border-white/20 hover:bg-white/10 transition-all group"
		>
			<h2 class="text-2xl font-semibold text-white mb-2 group-hover:text-blue-300 transition-colors">
				⚡ Commands
			</h2>
			<p class="text-gray-400">Browse all available bot commands</p>
		</a>

		<a
			href="/emojis"
			class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 hover:border-white/20 hover:bg-white/10 transition-all group"
		>
			<h2 class="text-2xl font-semibold text-white mb-2 group-hover:text-blue-300 transition-colors">
				😀 Emojis
			</h2>
			<p class="text-gray-400">View and copy custom server emojis</p>
		</a>

		<a
			href="/tags"
			class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 hover:border-white/20 hover:bg-white/10 transition-all group"
		>
			<h2 class="text-2xl font-semibold text-white mb-2 group-hover:text-blue-300 transition-colors">
				🏷️ Tags
			</h2>
			<p class="text-gray-400">Create and manage custom tags</p>
		</a>

		<a
			href="/history"
			class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 hover:border-white/20 hover:bg-white/10 transition-all group"
		>
			<h2 class="text-2xl font-semibold text-white mb-2 group-hover:text-blue-300 transition-colors">
				📜 History
			</h2>
			<p class="text-gray-400">Browse command execution history</p>
		</a>
	</div>
</div>
