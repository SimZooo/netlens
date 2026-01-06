<script lang="ts">
    import { PaneGroup, Pane, PaneResizer } from "paneforge";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import * as Infinitable from "svelte-infinitable";
    import type { NetworkPacket } from "$lib";
    import { onMount } from "svelte";

    let listening = $state(false);
    let packets: NetworkPacket[] = $state([]);

    listen<NetworkPacket>("packet_received", (e) => {
        let pkt = e.payload;
        pkt.last_protocol = pkt.layers.at(-1)?.protocol!;
        pkt.time = new Date(pkt.timestamp * 1000).toLocaleTimeString(); // Switch to time since execution?
        packets = [...packets, e.payload];
    });

    onMount(() => {});
</script>

<div class="h-screen w-screen bg-background grid grid-rows-[4em_auto] min-h-0">
    <div class="w-full border-b-textd border-b grid grid-rows-2">
        <div></div>
        <div class="p-1">
            <button
                title="toggle_listening"
                class="w-5 h-5"
                style:background-color={listening
                    ? "oklch(72.3% 0.219 149.579)"
                    : "oklch(63.7% 0.237 25.331)"}
                onclick={() => {
                    listening = !listening;
                    invoke("set_listen", { val: listening });
                }}
            >
            </button>
        </div>
    </div>

    <div class="flex-1 overflow-hidden text-textl">
        <PaneGroup direction="vertical" class="h-full">
            <Pane>
                <div class="h-full min-h-0 overflow-auto">
                    <Infinitable.Root
                        bind:items={packets}
                        rowHeight={28}
                        class="min-h-0"
                        ignoreInfinite={true}
                    >
                        {#snippet headers()}
                            <Infinitable.Header
                                header={{ label: "No." }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Time" }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Source" }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Destination" }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Protocol" }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Length" }}
                                class="text-left"
                            />
                            <Infinitable.Header
                                header={{ label: "Info" }}
                                class="text-left"
                            />
                        {/snippet}

                        {#snippet children({ index })}
                            {@const pkt = packets[index]}
                            <td>{index}</td>
                            <td>{pkt.time}</td>
                            <td>{pkt.src}</td>
                            <td>{pkt.dst}</td>
                            <td>{pkt.last_protocol}</td>
                            <td>15784</td>
                            <td>Yo</td>
                        {/snippet}
                    </Infinitable.Root>
                </div>
            </Pane>

            <PaneResizer class="w-full h-1 border-t-textd border-t" />
            <Pane defaultSize={25} />
        </PaneGroup>
    </div>
</div>

<style>
    Pane,
    Pane > div {
        min-height: 0;
    }

    td {
        white-space: nowrap;
        padding: 0.25rem 0.5rem;
    }
</style>
