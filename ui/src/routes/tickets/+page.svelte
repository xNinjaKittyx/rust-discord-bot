<script lang="ts">
	import { onMount } from 'svelte';

	interface EmbedConfig {
		title?: string;
		description?: string;
		url?: string;
		color?: number;
		author_name?: string;
		author_url?: string;
		author_icon_url?: string;
		footer_text?: string;
		footer_icon_url?: string;
		image_url?: string;
		thumbnail_url?: string;
	}

	interface ButtonConfig {
		label: string;
		emoji?: string;
	}

	interface ModalField {
		label: string;
		placeholder?: string;
		style: 'Short' | 'Paragraph';
	}

	interface ModalConfig {
		title_field: ModalField;
		description_field: ModalField;
	}

	interface TicketMenu {
		id: string;
		channel_id: number;
		message_id?: number;
		guild_id: number;
		category_id: number;
		embed_config: EmbedConfig;
		button_config: ButtonConfig;
		modal_config: ModalConfig;
	}

	interface Channel {
		id: string;
		name: string;
		guild_id: string;
		guild_name: string;
	}

	interface Category {
		id: string;
		name: string;
		guild_id: string;
		guild_name: string;
	}

	interface ChannelsResponse {
		channels: Channel[];
		categories: Category[];
	}

	interface TicketsResponse {
		tickets: TicketMenu[];
		count: number;
	}

	let tickets = $state<TicketMenu[]>([]);
	let channels = $state<Channel[]>([]);
	let categories = $state<Category[]>([]);
	let isLoading = $state(true);
	let error = $state('');

	// Modal state
	let showModal = $state(false);
	let modalMode = $state<'create' | 'edit'>('create');
	let currentTicket = $state<TicketMenu | null>(null);

	// Form fields
	let formId = $state('');
	let formChannelId = $state('');
	let formGuildId = $state('');
	let formCategoryId = $state('');

	// Embed config
	let embedTitle = $state('🎫 Support Tickets');
	let embedDescription = $state('Click the button below to create a support ticket.');
	let embedUrl = $state('');
	let embedColor = $state('#5865F2');
	let embedAuthorName = $state('');
	let embedAuthorUrl = $state('');
	let embedAuthorIcon = $state('');
	let embedFooterText = $state('');
	let embedFooterIcon = $state('');
	let embedImageUrl = $state('');
	let embedThumbnailUrl = $state('');

	// Button config
	let buttonLabel = $state('Create Ticket');
	let buttonEmoji = $state('🎫');

	// Modal config
	let modalTitleLabel = $state('Title');
	let modalTitlePlaceholder = $state('Brief description of your issue');
	let modalDescLabel = $state('Description');
	let modalDescPlaceholder = $state('Please provide details about your request');

	// Delete confirmation modal state
	let showDeleteModal = $state(false);
	let deleteIdPending = $state('');

	async function fetchTickets() {
		isLoading = true;
		error = '';
		try {
			const response = await fetch('/api/tickets');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: TicketsResponse = await response.json();
			tickets = data.tickets;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch tickets';
			console.error('Error fetching tickets:', e);
		} finally {
			isLoading = false;
		}
	}

	async function fetchChannels() {
		try {
			const response = await fetch('/api/channels');
			if (!response.ok) {
				throw new Error(`HTTP error! status: ${response.status}`);
			}
			const data: ChannelsResponse = await response.json();
			channels = data.channels;
			categories = data.categories;
		} catch (e) {
			console.error('Error fetching channels:', e);
		}
	}

	onMount(() => {
		fetchTickets();
		fetchChannels();
	});

	function openCreateModal() {
		modalMode = 'create';
		resetForm();
		showModal = true;
	}

	function openEditModal(ticket: TicketMenu) {
		modalMode = 'edit';
		currentTicket = ticket;
		loadTicketIntoForm(ticket);
		showModal = true;
	}

	function resetForm() {
		formId = '';
		formChannelId = '';
		formGuildId = '';
		formCategoryId = '';
		embedTitle = '🎫 Support Tickets';
		embedDescription = 'Click the button below to create a support ticket.';
		embedUrl = '';
		embedColor = '#5865F2';
		embedAuthorName = '';
		embedAuthorUrl = '';
		embedAuthorIcon = '';
		embedFooterText = '';
		embedFooterIcon = '';
		embedImageUrl = '';
		embedThumbnailUrl = '';
		buttonLabel = 'Create Ticket';
		buttonEmoji = '🎫';
		modalTitleLabel = 'Title';
		modalTitlePlaceholder = 'Brief description of your issue';
		modalDescLabel = 'Description';
		modalDescPlaceholder = 'Please provide details about your request';
	}

	function loadTicketIntoForm(ticket: TicketMenu) {
		formId = ticket.id;
		formChannelId = ticket.channel_id.toString();
		formGuildId = ticket.guild_id.toString();
		formCategoryId = ticket.category_id.toString();
		embedTitle = ticket.embed_config.title || '';
		embedDescription = ticket.embed_config.description || '';
		embedUrl = ticket.embed_config.url || '';
		embedColor = ticket.embed_config.color ? `#${ticket.embed_config.color.toString(16).padStart(6, '0')}` : '#5865F2';
		embedAuthorName = ticket.embed_config.author_name || '';
		embedAuthorUrl = ticket.embed_config.author_url || '';
		embedAuthorIcon = ticket.embed_config.author_icon_url || '';
		embedFooterText = ticket.embed_config.footer_text || '';
		embedFooterIcon = ticket.embed_config.footer_icon_url || '';
		embedImageUrl = ticket.embed_config.image_url || '';
		embedThumbnailUrl = ticket.embed_config.thumbnail_url || '';
		buttonLabel = ticket.button_config.label;
		buttonEmoji = ticket.button_config.emoji || '';
		modalTitleLabel = ticket.modal_config.title_field.label;
		modalTitlePlaceholder = ticket.modal_config.title_field.placeholder || '';
		modalDescLabel = ticket.modal_config.description_field.label;
		modalDescPlaceholder = ticket.modal_config.description_field.placeholder || '';
	}

	function closeModal() {
		showModal = false;
		currentTicket = null;
	}

	async function saveTicket() {
		if (!formId.trim() || !formChannelId || !formGuildId || !formCategoryId) {
			alert('ID, Channel, Guild, and Category are required');
			return;
		}

		const colorNum = parseInt(embedColor.replace('#', ''), 16);

		const ticketData = {
			id: formId,
			channel_id: formChannelId,
			guild_id: formGuildId,
			category_id: formCategoryId,
			embed_config: {
				title: embedTitle || null,
				description: embedDescription || null,
				url: embedUrl || null,
				color: colorNum || null,
				author_name: embedAuthorName || null,
				author_url: embedAuthorUrl || null,
				author_icon_url: embedAuthorIcon || null,
				footer_text: embedFooterText || null,
				footer_icon_url: embedFooterIcon || null,
				image_url: embedImageUrl || null,
				thumbnail_url: embedThumbnailUrl || null
			},
			button_config: {
				label: buttonLabel,
				emoji: buttonEmoji || null
			},
			modal_config: {
				title_field: {
					label: modalTitleLabel,
					placeholder: modalTitlePlaceholder || null,
					style: 'Short'
				},
				description_field: {
					label: modalDescLabel,
					placeholder: modalDescPlaceholder || null,
					style: 'Paragraph'
				}
			}
		};

		try {
			let response;
			if (modalMode === 'create') {
				response = await fetch('/api/tickets', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify(ticketData)
				});
			} else {
				response = await fetch(`/api/tickets/${formId}`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify(ticketData)
				});
			}

			if (!response.ok) {
				const errorData = await response.json();
				throw new Error(errorData.error || 'Failed to save ticket');
			}

			await fetchTickets();
			closeModal();
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to save ticket');
		}
	}

	function confirmDelete(id: string) {
		deleteIdPending = id;
		showDeleteModal = true;
	}

	async function deleteTicket() {
		try {
			const response = await fetch(`/api/tickets/${deleteIdPending}`, {
				method: 'DELETE'
			});

			if (!response.ok) {
				throw new Error('Failed to delete ticket');
			}

			await fetchTickets();
			showDeleteModal = false;
			deleteIdPending = '';
		} catch (e) {
			alert(e instanceof Error ? e.message : 'Failed to delete ticket');
		}
	}

	function getChannelName(channelId: number): string {
		const channel = channels.find(c => c.id === channelId.toString());
		return channel ? `#${channel.name} (${channel.guild_name})` : `Channel ${channelId}`;
	}

	function getCategoryName(categoryId: number): string {
		const category = categories.find(c => c.id === categoryId.toString());
		return category ? `${category.name} (${category.guild_name})` : `Category ${categoryId}`;
	}

	// When channel changes, auto-set guild
	$effect(() => {
		if (formChannelId) {
			const channel = channels.find(c => c.id === formChannelId);
			if (channel) {
				formGuildId = channel.guild_id;
			}
		}
	});
</script>

<div class="container">
	<div class="header">
		<h1>🎫 Ticket Menus</h1>
		<button onclick={openCreateModal} class="btn-primary">Create New Ticket Menu</button>
	</div>

	{#if isLoading}
		<div class="loading">Loading tickets...</div>
	{:else if error}
		<div class="error">{error}</div>
	{:else if tickets.length === 0}
		<div class="empty">
			<p>No ticket menus configured yet.</p>
			<button onclick={openCreateModal} class="btn-primary">Create Your First Ticket Menu</button>
		</div>
	{:else}
		<div class="tickets-grid">
			{#each tickets as ticket}
				<div class="ticket-card">
					<div class="ticket-header">
						<h3>{ticket.id}</h3>
						<div class="ticket-actions">
							<button onclick={() => openEditModal(ticket)} class="btn-edit">Edit</button>
							<button onclick={() => confirmDelete(ticket.id)} class="btn-delete">Delete</button>
						</div>
					</div>
					<div class="ticket-info">
						<div class="info-row">
							<span class="label">Channel:</span>
							<span>{getChannelName(ticket.channel_id)}</span>
						</div>
						<div class="info-row">
							<span class="label">Category:</span>
							<span>{getCategoryName(ticket.category_id)}</span>
						</div>
						<div class="info-row">
							<span class="label">Button:</span>
							<span>{ticket.button_config.emoji} {ticket.button_config.label}</span>
						</div>
						{#if ticket.message_id}
							<div class="info-row">
								<span class="label">Message ID:</span>
								<span class="message-id">{ticket.message_id}</span>
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Create/Edit Modal -->
{#if showModal}
	<div class="modal-overlay" onclick={closeModal}>
		<div class="modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>{modalMode === 'create' ? 'Create Ticket Menu' : 'Edit Ticket Menu'}</h2>
				<button onclick={closeModal} class="btn-close">×</button>
			</div>
			<div class="modal-body">
				<div class="form-section">
					<h3>Basic Settings</h3>
					<div class="form-group">
						<label for="id">Menu ID *</label>
						<input
							id="id"
							type="text"
							bind:value={formId}
							disabled={modalMode === 'edit'}
							placeholder="e.g., support-tickets"
						/>
					</div>
					<div class="form-group">
						<label for="channel">Channel *</label>
						<select id="channel" bind:value={formChannelId}>
							<option value="">Select a channel</option>
							{#each channels as channel}
								<option value={channel.id}>
									#{channel.name} ({channel.guild_name})
								</option>
							{/each}
						</select>
					</div>
					<div class="form-group">
						<label for="category">Category ID *</label>
						<input
							id="category"
							type="text"
							bind:value={formCategoryId}
							placeholder="Category ID (right-click category in Discord and Copy ID)"
						/>
					</div>
				</div>

				<div class="form-section">
					<h3>Embed Configuration</h3>
					<div class="form-group">
						<label for="embed-title">Title</label>
						<input id="embed-title" type="text" bind:value={embedTitle} />
					</div>
					<div class="form-group">
						<label for="embed-desc">Description</label>
						<textarea id="embed-desc" bind:value={embedDescription} rows="3"></textarea>
					</div>
					<div class="form-row">
						<div class="form-group">
							<label for="embed-color">Color</label>
							<input id="embed-color" type="color" bind:value={embedColor} />
						</div>
						<div class="form-group">
							<label for="embed-url">URL</label>
							<input id="embed-url" type="text" bind:value={embedUrl} placeholder="https://..." />
						</div>
					</div>
					<div class="form-row">
						<div class="form-group">
							<label for="embed-image">Image URL</label>
							<input id="embed-image" type="text" bind:value={embedImageUrl} placeholder="https://..." />
						</div>
						<div class="form-group">
							<label for="embed-thumb">Thumbnail URL</label>
							<input id="embed-thumb" type="text" bind:value={embedThumbnailUrl} placeholder="https://..." />
						</div>
					</div>
					<details>
						<summary>Advanced Embed Options</summary>
						<div class="form-group">
							<label for="embed-author-name">Author Name</label>
							<input id="embed-author-name" type="text" bind:value={embedAuthorName} />
						</div>
						<div class="form-row">
							<div class="form-group">
								<label for="embed-author-url">Author URL</label>
								<input id="embed-author-url" type="text" bind:value={embedAuthorUrl} placeholder="https://..." />
							</div>
							<div class="form-group">
								<label for="embed-author-icon">Author Icon URL</label>
								<input id="embed-author-icon" type="text" bind:value={embedAuthorIcon} placeholder="https://..." />
							</div>
						</div>
						<div class="form-row">
							<div class="form-group">
								<label for="embed-footer-text">Footer Text</label>
								<input id="embed-footer-text" type="text" bind:value={embedFooterText} />
							</div>
							<div class="form-group">
								<label for="embed-footer-icon">Footer Icon URL</label>
								<input id="embed-footer-icon" type="text" bind:value={embedFooterIcon} placeholder="https://..." />
							</div>
						</div>
					</details>
				</div>

				<div class="form-section">
					<h3>Button Configuration</h3>
					<div class="form-row">
						<div class="form-group">
							<label for="button-label">Label *</label>
							<input id="button-label" type="text" bind:value={buttonLabel} />
						</div>
						<div class="form-group">
							<label for="button-emoji">Emoji</label>
							<input id="button-emoji" type="text" bind:value={buttonEmoji} placeholder="🎫" />
						</div>
					</div>
				</div>

				<div class="form-section">
					<h3>Modal Configuration</h3>
					<div class="form-group">
						<label for="modal-title-label">Title Field Label *</label>
						<input id="modal-title-label" type="text" bind:value={modalTitleLabel} />
					</div>
					<div class="form-group">
						<label for="modal-title-placeholder">Title Field Placeholder</label>
						<input id="modal-title-placeholder" type="text" bind:value={modalTitlePlaceholder} />
					</div>
					<div class="form-group">
						<label for="modal-desc-label">Description Field Label *</label>
						<input id="modal-desc-label" type="text" bind:value={modalDescLabel} />
					</div>
					<div class="form-group">
						<label for="modal-desc-placeholder">Description Field Placeholder</label>
						<input id="modal-desc-placeholder" type="text" bind:value={modalDescPlaceholder} />
					</div>
				</div>
			</div>
			<div class="modal-footer">
				<button onclick={closeModal} class="btn-secondary">Cancel</button>
				<button onclick={saveTicket} class="btn-primary">
					{modalMode === 'create' ? 'Create' : 'Update'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
	<div class="modal-overlay" onclick={() => (showDeleteModal = false)}>
		<div class="modal modal-small" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<h2>Confirm Delete</h2>
			</div>
			<div class="modal-body">
				<p>Are you sure you want to delete ticket menu <strong>{deleteIdPending}</strong>?</p>
				<p class="warning">This action cannot be undone.</p>
			</div>
			<div class="modal-footer">
				<button onclick={() => (showDeleteModal = false)} class="btn-secondary">Cancel</button>
				<button onclick={deleteTicket} class="btn-delete">Delete</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	h1 {
		font-size: 2rem;
		font-weight: 700;
		color: #fff;
		margin: 0;
	}

	.loading,
	.error,
	.empty {
		text-align: center;
		padding: 3rem;
		color: #b9bbbe;
	}

	.error {
		color: #f04747;
	}

	.empty p {
		margin-bottom: 1rem;
		font-size: 1.1rem;
	}

	.tickets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.ticket-card {
		background: #2f3136;
		border-radius: 8px;
		padding: 1.5rem;
		border: 1px solid #202225;
	}

	.ticket-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.ticket-header h3 {
		margin: 0;
		color: #fff;
		font-size: 1.25rem;
	}

	.ticket-actions {
		display: flex;
		gap: 0.5rem;
	}

	.ticket-info {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.info-row {
		display: flex;
		gap: 0.5rem;
		font-size: 0.9rem;
		color: #b9bbbe;
	}

	.info-row .label {
		font-weight: 600;
		color: #8e9297;
		min-width: 80px;
	}

	.message-id {
		font-family: 'Courier New', monospace;
		font-size: 0.85rem;
	}

	.btn-primary,
	.btn-secondary,
	.btn-edit,
	.btn-delete,
	.btn-close {
		padding: 0.5rem 1rem;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn-primary {
		background: #5865f2;
		color: white;
	}

	.btn-primary:hover {
		background: #4752c4;
	}

	.btn-secondary {
		background: #4f545c;
		color: white;
	}

	.btn-secondary:hover {
		background: #5d6269;
	}

	.btn-edit {
		background: #3ba55d;
		color: white;
		padding: 0.4rem 0.8rem;
		font-size: 0.9rem;
	}

	.btn-edit:hover {
		background: #2d7d46;
	}

	.btn-delete {
		background: #ed4245;
		color: white;
		padding: 0.4rem 0.8rem;
		font-size: 0.9rem;
	}

	.btn-delete:hover {
		background: #c03537;
	}

	.btn-close {
		background: transparent;
		color: #b9bbbe;
		font-size: 1.5rem;
		padding: 0;
		width: 2rem;
		height: 2rem;
	}

	.btn-close:hover {
		color: #fff;
		background: #4f545c;
	}

	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.85);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: #36393f;
		border-radius: 8px;
		width: 90%;
		max-width: 700px;
		max-height: 90vh;
		overflow-y: auto;
	}

	.modal-small {
		max-width: 450px;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem;
		border-bottom: 1px solid #2f3136;
	}

	.modal-header h2 {
		margin: 0;
		color: #fff;
		font-size: 1.5rem;
	}

	.modal-body {
		padding: 1.5rem;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		padding: 1.5rem;
		border-top: 1px solid #2f3136;
	}

	.form-section {
		margin-bottom: 2rem;
	}

	.form-section h3 {
		color: #fff;
		margin-top: 0;
		margin-bottom: 1rem;
		font-size: 1.1rem;
	}

	.form-group {
		margin-bottom: 1rem;
		flex: 1;
	}

	.form-row {
		display: flex;
		gap: 1rem;
	}

	label {
		display: block;
		margin-bottom: 0.5rem;
		color: #b9bbbe;
		font-weight: 500;
		font-size: 0.9rem;
	}

	input[type='text'],
	input[type='color'],
	select,
	textarea {
		width: 100%;
		padding: 0.75rem;
		background: #202225;
		border: 1px solid #2f3136;
		border-radius: 4px;
		color: #dcddde;
		font-size: 1rem;
		font-family: inherit;
	}

	input[type='color'] {
		height: 45px;
		cursor: pointer;
	}

	input:focus,
	select:focus,
	textarea:focus {
		outline: none;
		border-color: #5865f2;
	}

	input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	details {
		margin-top: 1rem;
	}

	summary {
		color: #00aff4;
		cursor: pointer;
		user-select: none;
		margin-bottom: 1rem;
	}

	summary:hover {
		color: #00c9ff;
	}

	.warning {
		color: #faa61a;
		font-size: 0.9rem;
	}
</style>
