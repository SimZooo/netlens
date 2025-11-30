<script>
	import favicon from '$lib/assets/favicon.svg';
	let { children } = $props();
	import { get_interfaces, interface_index } from '$lib/index.ts';
	import Time from "svelte-time"
	import "../app.css";
    import { onMount } from 'svelte';
	let interfaces = $state([]);

	const start_time = new Date();
	onMount(async () => {
		interfaces = await get_interfaces();
	})
</script>

<main class="h-screen w-screen p-2 grid grid-rows-[auto_1fr]">
	<div class="flex justify-between">
		<div class="flex flex-row">
			<h1 class="text-3xl p-3">NetLens</h1>
			<select name="intercae" id="" bind:value={$interface_index}>
				{#each interfaces as iface, i}
					<option value="{i}">
						{iface}
					</option>
				{/each}
			</select>
		</div>
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

