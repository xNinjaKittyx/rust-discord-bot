<script lang="ts">
	import { onMount } from 'svelte';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';

	interface Command {
		name: string;
		description: string;
		category: string;
		usage: string;
	}

	interface CommandsResponse {
		commands: Command[];
		count: number;
	}

	let commands = $state<Command[]>([]);
	let filteredCommands = $state<Command[]>([]);
	let searchText = $state('');
	let selectedCategory = $state('All');
	let categories = $state<string[]>(['All']);
	let isLoading = $state(true);

	// Configure marked to use GitHub Flavored Markdown
	marked.setOptions({
		breaks: true,
		gfm: true
	});

	function renderMarkdown(markdown: string): string {
		const html = marked(markdown) as string;
		return DOMPurify.sanitize(html);
	}

	async function fetchCommands() {
		isLoading = true;
		try {
			const response = await fetch('/api/commands');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: CommandsResponse = await response.json();
			commands = data.commands;
			filteredCommands = data.commands;

			// Extract unique categories
			const uniqueCategories = [...new Set(data.commands.map((cmd) => cmd.category))];
			categories = ['All', ...uniqueCategories.sort()];
		} catch (e) {
			console.error('Error fetching commands:', e);
		} finally {
			isLoading = false;
		}
	}

	function handleFilter() {
		let filtered = commands;

		// Filter by category
		if (selectedCategory !== 'All') {
			filtered = filtered.filter((cmd) => cmd.category === selectedCategory);
		}

		// Filter by search text
		if (searchText.trim()) {
			const search = searchText.toLowerCase();
			filtered = filtered.filter(
				(cmd) =>
					cmd.name.toLowerCase().includes(search) ||
					cmd.description.toLowerCase().includes(search) ||
					cmd.usage.toLowerCase().includes(search)
			);
		}

		filteredCommands = filtered;
	}

	function getCategoryColor(category: string): string {
		const colors: Record<string, string> = {
			Basic: 'bg-blue-500/20 text-blue-300 border-blue-500/30',
			Random: 'bg-purple-500/20 text-purple-300 border-purple-500/30',
			Entertainment: 'bg-pink-500/20 text-pink-300 border-pink-500/30',
			Utility: 'bg-green-500/20 text-green-300 border-green-500/30',
			Music: 'bg-orange-500/20 text-orange-300 border-orange-500/30',
			AI: 'bg-cyan-500/20 text-cyan-300 border-cyan-500/30',
			Streams: 'bg-red-500/20 text-red-300 border-red-500/30',
			Permissions: 'bg-yellow-500/20 text-yellow-300 border-yellow-500/30',
			Anime: 'bg-indigo-500/20 text-indigo-300 border-indigo-500/30'
		};
		return colors[category] || 'bg-gray-500/20 text-gray-300 border-gray-500/30';
	}

	onMount(() => {
		fetchCommands();
	});

	$effect(() => {
		handleFilter();
	});
</script>

<div class="max-w-7xl mx-auto">
	<h1 class="text-3xl font-bold text-white mb-6">Commands</h1>

	<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 mb-6">
		<div class="flex flex-col md:flex-row gap-4">
			<input
				type="text"
				bind:value={searchText}
				placeholder="Search commands..."
				class="flex-1 px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500"
			/>
			<select
				bind:value={selectedCategory}
				class="px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white"
			>
				{#each categories as category}
					<option value={category}>{category}</option>
				{/each}
			</select>
		</div>
		<div class="mt-4 text-sm text-gray-400">
			Showing {filteredCommands.length} of {commands.length} commands
		</div>
	</div>

	{#if isLoading}
		<div class="text-center text-gray-400 py-12">Loading commands...</div>
	{:else if filteredCommands.length === 0}
		<div class="text-center text-gray-500 py-12">No commands found</div>
	{:else}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
			{#each filteredCommands as command (command.name)}
				<div
					class="bg-white/5 backdrop-blur-sm border border-white/10 rounded-lg p-5 hover:bg-white/10 transition-all hover:border-white/20"
				>
					<div class="flex items-start justify-between mb-3">
						<div class="flex-1">
							<h3 class="text-xl font-semibold text-white mb-1">/{command.name}</h3>
							<code
								class="text-xs font-mono text-gray-400 bg-black/40 px-2 py-1 rounded border border-white/10"
							>
								{command.usage}
							</code>
						</div>
						<span
							class="px-3 py-1 text-xs font-medium rounded-full border {getCategoryColor(
								command.category
							)}"
						>
							{command.category}
						</span>
					</div>
					<div class="text-gray-300 text-sm leading-relaxed prose prose-invert prose-sm max-w-none">
						{@html renderMarkdown(command.description)}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	:global(.prose) {
		color: rgb(209 213 219);
	}
	:global(.prose strong) {
		color: rgb(243 244 246);
		font-weight: 600;
	}
	:global(.prose code) {
		color: rgb(147 197 253);
		background-color: rgba(0, 0, 0, 0.4);
		padding: 0.125rem 0.375rem;
		border-radius: 0.25rem;
		font-size: 0.875em;
	}
	:global(.prose ul) {
		list-style-type: disc;
		padding-left: 1.5rem;
		margin-top: 0.5rem;
	}
	:global(.prose li) {
		margin-top: 0.25rem;
	}
	:global(.prose p) {
		margin-top: 0.5rem;
		margin-bottom: 0.5rem;
	}
	:global(.prose p:first-child) {
		margin-top: 0;
	}
</style>
