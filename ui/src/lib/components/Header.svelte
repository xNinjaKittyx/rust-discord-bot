<script lang="ts">
	import { page } from '$app/stores';

	const navItems = [
		{ name: 'Logging', href: '/logging' },
		{ name: 'Commands', href: '/commands' },
		{ name: 'Emojis', href: '/emojis' },
		{ name: 'Tags', href: '/tags' },
		{ name: 'Tickets', href: '/tickets' },
		{ name: 'History', href: '/history' },
		{ name: 'AYDY', href: '/aydy' }
	];
</script>

<header class="border-b border-gray-800/50 backdrop-blur-sm bg-black/40 sticky top-0 z-50">
	<div class="container mx-auto px-6">
		<div class="flex items-center justify-between h-16">
			<a
				href="/"
				class="text-lg font-semibold tracking-tight bg-gradient-to-r from-white to-gray-400 bg-clip-text text-transparent hover:from-gray-100 hover:to-gray-300 transition-all"
			>
				Urfmode RinBot
			</a>
			<nav class="flex items-center gap-1">
				{#each navItems as item}
					<a
						href={item.href}
						class="px-4 py-2 text-sm font-medium transition-all relative group {$page.url
							.pathname === item.href
							? 'text-white'
							: 'text-gray-400 hover:text-white'}"
					>
						{item.name}
						{#if $page.url.pathname === item.href}
							<div
								class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-blue-500 to-purple-500"
							></div>
						{:else}
							<div
								class="absolute bottom-0 left-0 right-0 h-0.5 bg-gradient-to-r from-blue-500 to-purple-500 opacity-0 group-hover:opacity-100 transition-opacity"
							></div>
						{/if}
					</a>
				{/each}
			</nav>
		</div>
	</div>
</header>
