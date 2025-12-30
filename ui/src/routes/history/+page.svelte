<script lang="ts">
	import { onMount } from 'svelte';

	interface HistoryEntry {
		timestamp: string;
		user: string;
		user_id: string;
		command: string;
		full_invocation?: string;
		guild: string;
		channel: string;
		success: boolean;
		error?: string;
	}

	interface HistoryResponse {
		history: HistoryEntry[];
		count: number;
	}

	let history = $state<HistoryEntry[]>([]);
	let isLoading = $state(true);
	let error = $state('');
	let autoRefresh = $state(true);
	let intervalId: ReturnType<typeof setInterval> | undefined;
	let expandedEntries = $state<Set<string>>(new Set());

	function toggleExpand(key: string) {
		const newSet = new Set(expandedEntries);
		if (newSet.has(key)) {
			newSet.delete(key);
		} else {
			newSet.add(key);
		}
		expandedEntries = newSet;
	}

	async function fetchHistory(showLoading = false) {
		if (showLoading) {
			isLoading = true;
		}
		error = '';
		try {
			const response = await fetch('/api/history');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: HistoryResponse = await response.json();
			history = data.history;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch history';
			console.error('Error fetching history:', e);
		} finally {
			isLoading = false;
		}
	}

	function formatTimestamp(timestamp: string): string {
		try {
			const date = new Date(timestamp);
			return date.toLocaleString();
		} catch {
			return timestamp;
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			intervalId = setInterval(fetchHistory, 5000);
		} else {
			if (intervalId) {
				clearInterval(intervalId);
				intervalId = undefined;
			}
		}
	}

	$effect(() => {
		fetchHistory(true);
		if (autoRefresh) {
			intervalId = setInterval(() => fetchHistory(false), 5000);
		}
		return () => {
			if (intervalId) clearInterval(intervalId);
		};
	});
</script>

<div class="max-w-6xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-3xl font-bold text-white">Command History</h1>
		<button
			onclick={toggleAutoRefresh}
			class="px-4 py-2 {autoRefresh
				? 'bg-green-600 hover:bg-green-700'
				: 'bg-gray-600 hover:bg-gray-700'} text-white rounded-lg transition-colors font-medium text-sm"
		>
			{autoRefresh ? '✓ Auto-refresh' : '⏸ Paused'}
		</button>
	</div>

	{#if error}
		<div class="bg-red-500/10 border border-red-500/50 text-red-400 rounded-lg p-4 mb-4">
			<strong>Error:</strong>
			{error}
		</div>
	{/if}

	{#if isLoading}
		<div class="text-center text-gray-400 py-12">Loading history...</div>
	{:else if history.length === 0}
		<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-12 text-center">
			<p class="text-gray-400 text-lg">No command history found</p>
			<p class="text-gray-500 text-sm mt-2">Commands will appear here as they are executed</p>
		</div>
	{:else}
		<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-black/40 border-b border-white/10">
						<tr>
							<th
								class="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
							>
								Timestamp
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
							>
								User
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
							>
								Command
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
							>
								Guild
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider"
							>
								Status
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-white/10">
						{#each history as entry (entry.timestamp + entry.user_id)}
							{@const entryKey = entry.timestamp + entry.user_id}
							{@const isExpanded = expandedEntries.has(entryKey)}
							<tr class="hover:bg-white/5 transition-colors">
								<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
									{formatTimestamp(entry.timestamp)}
								</td>
								<td class="px-6 py-4 whitespace-nowrap text-sm">
									<div class="text-white">{entry.user}</div>
									<div class="text-gray-500 text-xs">{entry.user_id}</div>
								</td>
								<td class="px-6 py-4 text-sm">
									<button
										onclick={() => toggleExpand(entryKey)}
										class="text-left w-full font-mono text-blue-300 hover:text-blue-200 transition-colors"
									>
										<div class="flex items-center gap-2">
											{#if entry.full_invocation && entry.full_invocation !== `/${entry.command}`}
												<svg
													class="w-3 h-3 transform transition-transform flex-shrink-0 {isExpanded ? 'rotate-90' : ''}"
													viewBox="0 0 10 10"
													fill="currentColor"
												>
													<path d="M2,1 L8,5 L2,9 Z" />
												</svg>
											{/if}
											<span class={entry.full_invocation && entry.full_invocation !== `/${entry.command}` ? '' : 'ml-5'}>
												/{entry.command}
											</span>
										</div>
										{#if isExpanded && entry.full_invocation}
											<div class="mt-2 ml-5 text-gray-400 text-xs bg-black/40 rounded px-2 py-1 border border-white/10">
												{entry.full_invocation}
											</div>
										{/if}
									</button>
								</td>
								<td class="px-6 py-4 whitespace-nowrap text-sm">
									<div class="text-white">{entry.guild}</div>
									<div class="text-gray-500 text-xs">{entry.channel}</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<span
										class="px-2 py-1 text-xs font-medium rounded {entry.success
											? 'bg-green-500/20 text-green-300'
											: 'bg-red-500/20 text-red-300'}"
									>
										{entry.success ? 'Success' : 'Failed'}
									</span>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
		
		<div class="mt-4 text-center text-gray-400 text-sm">
			Showing {history.length} most recent command{history.length !== 1 ? 's' : ''}
		</div>
	{/if}
</div>

