<script lang="ts">
	import { onMount } from 'svelte';

	interface LogsResponse {
		logs: string[];
		count: number;
	}

	let logs = $state<string[]>([]);
	let filteredLogs = $state<string[]>([]);
	let filterText = $state('');
	let isLoading = $state(true);
	let error = $state('');
	let autoRefresh = $state(true);
	let intervalId: ReturnType<typeof setInterval> | undefined;

	async function fetchLogs() {
		isLoading = true;
		error = '';
		try {
			const response = await fetch('/api/logs');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: LogsResponse = await response.json();
			logs = data.logs;
			filteredLogs = data.logs;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch logs';
			console.error('Error fetching logs:', e);
		} finally {
			isLoading = false;
		}
	}

	function handleFilter() {
		if (!filterText.trim()) {
			filteredLogs = logs;
		} else {
			filteredLogs = logs.filter((log) =>
				log.toLowerCase().includes(filterText.toLowerCase())
			);
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			intervalId = setInterval(fetchLogs, 5000);
		} else {
			if (intervalId) {
				clearInterval(intervalId);
				intervalId = undefined;
			}
		}
	}

	function getLogColor(log: string): string {
		if (log.includes('[INFO]')) return 'text-green-400';
		if (log.includes('[WARN]')) return 'text-yellow-400';
		if (log.includes('[ERROR]')) return 'text-red-400';
		if (log.includes('[DEBUG]')) return 'text-blue-400';
		if (log.includes('[TRACE]')) return 'text-gray-500';
		return 'text-gray-300';
	}

	onMount(() => {
		fetchLogs();
		// Auto-refresh every 5 seconds
		intervalId = setInterval(fetchLogs, 5000);
		return () => {
			if (intervalId) clearInterval(intervalId);
		};
	});
</script>

<div class="max-w-6xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-3xl font-bold text-white">Logging</h1>
		<button
			onclick={toggleAutoRefresh}
			class="px-4 py-2 bg-white/10 text-white rounded-lg hover:bg-white/20 transition-colors font-medium text-sm"
		>
			Auto-refresh: {autoRefresh ? 'ON' : 'OFF'}
		</button>
	</div>

	<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6">
		<div class="mb-4">
			<input
				type="text"
				bind:value={filterText}
				onkeyup={handleFilter}
				placeholder="Filter logs..."
				class="w-full px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500"
			/>
		</div>

		{#if error}
			<div class="bg-red-500/10 border border-red-500/50 text-red-400 rounded-lg p-4 mb-4">
				<strong>Error:</strong>
				{error}
			</div>
		{/if}

		<div
			class="bg-black/60 border border-white/10 rounded-lg p-4 font-mono text-sm min-h-96 max-h-[calc(100vh-20rem)] overflow-y-auto"
		>
			{#if isLoading && logs.length === 0}
				<div class="text-gray-400">Loading logs...</div>
			{:else if filteredLogs.length === 0}
				<div class="text-gray-500">No logs found</div>
			{:else}
				{#each filteredLogs as log (log)}
					<div class={getLogColor(log)}>{log}</div>
				{/each}
			{/if}
		</div>

		<div class="mt-4 text-sm text-gray-400">
			Showing {filteredLogs.length} of {logs.length} log entries
		</div>
	</div>
</div>
