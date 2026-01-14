<script lang="ts">
    import type { NetworkPacket } from "$lib";
    import {
        LucideArrowRight,
        LucideChevronDown,
        LucideChevronRight,
    } from "lucide-svelte";
    import { onMount } from "svelte";

    let { packet }: { packet: NetworkPacket } = $props();
    let expanded_status: boolean[] = $state([]);

    $effect(() => {
        expanded_status = Array.from(
            { length: packet.layers.length },
            () => false,
        );
    });
</script>

<div class="size-full p-2 overflow-y-auto">
    <p>Packet: {packet.id}</p>
    {#each packet.layers as layer, i}
        <div class="w-full h-fit flex flex-col">
            <button
                onclick={() => (expanded_status[i] = !expanded_status[i])}
                class="text-left flex flex-row bg-foreground"
            >
                {#if expanded_status[i]}
                    <LucideChevronDown size="22" />{layer.name}
                {:else}
                    <LucideChevronRight size="22" />{layer.name}
                {/if}
            </button>
            {#if expanded_status[i]}
                <div class="size-full pl-4">
                    {#each layer.fields as field}
                        <p>{field.name}: {field.value}</p>
                    {/each}
                </div>
            {/if}
        </div>
    {/each}
</div>

<style>
</style>
