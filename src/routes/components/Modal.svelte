<script>
    let { toggled, children, rect } = $props();
    import Portal from "svelte-portal";
    let modal;
</script>

{#if toggled}
<Portal target="main">
    <div class="absolute bg-[#162029] border-[#B6BDBD] border w-40 h-15 rounded flex flex-col justify-center text-center" style="left: {rect.left + rect.width/2}px; top: {rect.top + rect.height + 8}px; transform: translateX(-50%);" bind:this={modal}>
        {@render children()}
    </div>
</Portal>
{/if}

<style>

</style>