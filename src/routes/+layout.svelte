<script>
	import favicon from '$lib/assets/favicon.svg';
	let { children } = $props();
	import Time from "svelte-time"
	import "../app.css";

	const start_time = new Date();
</script>

<main class="h-screen w-screen p-2 grid grid-rows-[auto_1fr]">
	<div class="flex justify-between">
		<h1 class="text-3xl p-3">NetLens</h1>
		<h1 class="text-sm">Started: <Time timeStamp={start_time} format="dddd @ h:mm:ss A · MMMM D, YYYY" /></h1>
	</div>
	<div>
		{@render children()}
	</div>
</main>

<style>
	* {
		background-color: #0C141B;
		color: #B6BDBD;
		font-family: "Noto Sans";
		scrollbar-width: none;
        -ms-overflow-style: none;
		overflow: hidden;
	}

</style>

