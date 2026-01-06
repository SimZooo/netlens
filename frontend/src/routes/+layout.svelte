<script lang="ts">
    import favicon from "$lib/assets/favicon.svg";
    import "../app.css";

    let { children } = $props();
</script>

<svelte:head>
    <link rel="icon" href={favicon} />
</svelte:head>

{@render children()}

<style>
    * {
        font-family: "0xProto", sans-serif;
    }
</style>
