<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import Header from '$lib/components/Header.svelte';
	import Footer from '$lib/components/Footer.svelte';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="min-h-screen text-white relative flex flex-col">
	<div
		class="fixed inset-0 bg-cover bg-center -z-10"
		style="background-image: url('https://random.urfmode.moe'); filter: blur(8px); transform: scale(1.1);"
	></div>
	<div class="fixed inset-0 bg-black/60 -z-10"></div>
	<Header />
	<main class="container mx-auto px-6 py-12 flex-1">{@render children()}</main>
	<Footer />
</div>
