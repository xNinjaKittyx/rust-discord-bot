<script lang="ts">
	import { onMount } from 'svelte';

	interface Tag {
		key: string;
		value: string;
	}

	interface TagsResponse {
		tags: Tag[];
		count: number;
	}

	let tags = $state<Tag[]>([]);
	let filteredTags = $state<Tag[]>([]);
	let searchText = $state('');
	let expandedKey = $state<string | null>(null);
	let isLoading = $state(true);
	let error = $state('');

	// Modal state
	let showModal = $state(false);
	let modalMode = $state<'create' | 'edit'>('create');
	let modalKey = $state('');
	let modalValue = $state('');
	let originalKey = $state('');

	// Delete confirmation modal state
	let showDeleteModal = $state(false);
	let deleteKeyPending = $state('');

	async function fetchTags() {
		isLoading = true;
		error = '';
		try {
			const response = await fetch('/api/tags');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: TagsResponse = await response.json();
			tags = data.tags;
			filteredTags = data.tags;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch tags';
			console.error('Error fetching tags:', e);
		} finally {
			isLoading = false;
		}
	}

	function handleFilter() {
		if (!searchText.trim()) {
			filteredTags = tags;
		} else {
			const search = searchText.toLowerCase();
			filteredTags = tags.filter(
				(tag) =>
					tag.key.toLowerCase().includes(search) || tag.value.toLowerCase().includes(search)
			);
		}
	}

	function toggleExpand(key: string) {
		expandedKey = expandedKey === key ? null : key;
	}

	function openCreateModal() {
		modalMode = 'create';
		modalKey = '';
		modalValue = '';
		originalKey = '';
		showModal = true;
	}

	function openEditModal(tag: Tag) {
		modalMode = 'edit';
		modalKey = tag.key;
		modalValue = tag.value;
		originalKey = tag.key;
		showModal = true;
	}

	function closeModal() {
		showModal = false;
		modalKey = '';
		modalValue = '';
		originalKey = '';
	}

	async function saveTag() {
		if (!modalKey.trim() || !modalValue.trim()) {
			alert('Both key and value are required');
			return;
		}

		try {
			let response;
			if (modalMode === 'create') {
				response = await fetch('/api/tags', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ key: modalKey, value: modalValue })
				});
			} else {
				response = await fetch(`/api/tags/${encodeURIComponent(originalKey)}`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ key: modalKey, value: modalValue })
				});
			}

			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.error || 'Failed to save tag');
			}

			closeModal();
			await fetchTags();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save tag';
		}
	}

	function openDeleteModal(key: string) {
		deleteKeyPending = key;
		showDeleteModal = true;
	}

	function closeDeleteModal() {
		showDeleteModal = false;
		deleteKeyPending = '';
	}

	async function confirmDelete() {
		try {
			const response = await fetch(`/api/tags/${encodeURIComponent(deleteKeyPending)}`, {
				method: 'DELETE'
			});

			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.error || 'Failed to delete tag');
			}

			closeDeleteModal();
			await fetchTags();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete tag';
			closeDeleteModal();
		}
	}

	onMount(() => {
		fetchTags();
	});

	$effect(() => {
		handleFilter();
	});
</script>

<div class="max-w-6xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-3xl font-bold text-white">Tags</h1>
		<button
			onclick={openCreateModal}
			class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium text-sm"
		>
			+ New Tag
		</button>
	</div>

	<div class="bg-white/5 backdrop-blur-sm rounded-lg shadow-lg border border-white/10 p-6 mb-6">
		<input
			type="text"
			bind:value={searchText}
			placeholder="Search tags..."
			class="w-full px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500"
		/>
		<div class="mt-4 text-sm text-gray-400">
			Showing {filteredTags.length} of {tags.length} tags
		</div>
	</div>

	{#if error}
		<div class="bg-red-500/10 border border-red-500/50 text-red-400 rounded-lg p-4 mb-4">
			<strong>Error:</strong>
			{error}
		</div>
	{/if}

	{#if isLoading}
		<div class="text-center text-gray-400 py-12">Loading tags...</div>
	{:else if filteredTags.length === 0}
		<div class="text-center text-gray-500 py-12">No tags found</div>
	{:else}
		<div class="space-y-3">
			{#each filteredTags as tag (tag.key)}
				<div
					class="bg-white/5 backdrop-blur-sm border border-white/10 rounded-lg overflow-hidden hover:border-white/20 transition-all"
				>
					<button
						onclick={() => toggleExpand(tag.key)}
						class="w-full px-5 py-4 text-left flex items-center justify-between hover:bg-white/5 transition-colors"
					>
						<div class="flex items-center gap-3 flex-1">
							<svg
								class="w-3 h-3 transform transition-transform {expandedKey === tag.key
									? 'rotate-90'
									: ''}"
								viewBox="0 0 10 10"
								fill="white"
							>
								<path d="M2,1 L8,5 L2,9 Z" />
							</svg>
							<span class="font-semibold text-white">{tag.key}</span>
							{#if expandedKey !== tag.key}
								<span class="text-gray-500 text-sm truncate">
									{tag.value.slice(0, 100)}{tag.value.length > 100 ? '...' : ''}
								</span>
							{/if}
						</div>
					</button>

					{#if expandedKey === tag.key}
						<div class="px-5 pb-4 border-t border-white/10">
							<div class="pt-4 mb-4">
								<pre
									class="bg-black/40 border border-white/10 rounded-lg p-4 text-gray-300 text-sm whitespace-pre-wrap break-words">{tag.value}</pre>
							</div>
							<div class="flex gap-2">
								<button
									onclick={() => openEditModal(tag)}
									class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm font-medium"
								>
									Edit
								</button>
								<button
									onclick={() => openDeleteModal(tag.key)}
									class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm font-medium"
								>
									Delete
								</button>
							</div>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Modal -->
{#if showModal}
	<div
		class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200"
		onclick={closeModal}
	>
		<div
			class="bg-gray-900 border border-white/20 rounded-lg shadow-2xl max-w-2xl w-full animate-in zoom-in duration-300"
			style="transform-origin: left center;"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="px-6 py-4 border-b border-white/10 flex items-center justify-between">
				<h2 class="text-xl font-bold text-white">
					{modalMode === 'create' ? 'Create New Tag' : 'Edit Tag'}
				</h2>
				<button
					onclick={closeModal}
					class="text-gray-400 hover:text-white transition-colors text-2xl leading-none"
				>
					×
				</button>
			</div>

			<div class="p-6 space-y-4">
				<div>
					<label for="tag-key" class="block text-sm font-medium text-gray-300 mb-2">
						Key
					</label>
					<input
						id="tag-key"
						type="text"
						bind:value={modalKey}
						placeholder="Enter tag key..."
						class="w-full px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500"
					/>
				</div>

				<div>
					<label for="tag-value" class="block text-sm font-medium text-gray-300 mb-2">
						Value
					</label>
					<textarea
						id="tag-value"
						bind:value={modalValue}
						placeholder="Enter tag value..."
						rows="8"
						class="w-full px-4 py-2 bg-black/40 border border-white/20 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-white placeholder-gray-500 resize-none"
					></textarea>
				</div>
			</div>

			<div class="px-6 py-4 border-t border-white/10 flex justify-end gap-3">
				<button
					onclick={closeModal}
					class="px-4 py-2 bg-white/10 text-white rounded-lg hover:bg-white/20 transition-colors font-medium"
				>
					Cancel
				</button>
				<button
					onclick={saveTag}
					class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
				>
					{modalMode === 'create' ? 'Create' : 'Save'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
	<div
		class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4 animate-in fade-in duration-200"
		onclick={closeDeleteModal}
	>
		<div
			class="bg-gray-900 border border-white/20 rounded-lg shadow-2xl max-w-md w-full animate-in zoom-in duration-300"
			style="transform-origin: left center;"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="px-6 py-4 border-b border-white/10">
				<h2 class="text-xl font-bold text-white">Confirm Delete</h2>
			</div>

			<div class="p-6">
				<p class="text-gray-300">
					Are you sure you want to delete the tag <strong class="text-white">"{deleteKeyPending}"</strong
					>? This action cannot be undone.
				</p>
			</div>

			<div class="px-6 py-4 border-t border-white/10 flex justify-end gap-3">
				<button
					onclick={closeDeleteModal}
					class="px-4 py-2 bg-white/10 text-white rounded-lg hover:bg-white/20 transition-colors font-medium"
				>
					Cancel
				</button>
				<button
					onclick={confirmDelete}
					class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors font-medium"
				>
					Delete
				</button>
			</div>
		</div>
	</div>
{/if}
