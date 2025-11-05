<script>
    let { children, title, align } = $props();
    let rendered = $state(() => {});

    $effect(() => {
		rendered = children;
	});

</script>

<div class="bg-[#162029] main border-2 border-[#162029] rounded">
    <h1 class="text-lg p-2">
        {title}
    </h1>
    <div style="justify-content: {align}" class="bg-[#121B23] flex rounded overflow-x-auto">
        {@render rendered()}
    </div>
</div>

<style>
</style>