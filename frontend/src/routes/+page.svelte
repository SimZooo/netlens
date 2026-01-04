<script lang="ts">
    import { PaneGroup, Pane, PaneResizer } from "paneforge";
    import { get_top_layer, type NetworkPacket } from "$lib";
    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    let listening = $state(false);

    let packets: NetworkPacket[] = $state([]);
    listen<NetworkPacket>("packet_received", (e) => {
        packets.push(e.payload);
    });
</script>

<div class="h-screen w-screen bg-background grid grid-rows-[4em_auto] min-h-0">
    <div class="w-full border-b-textd border-b grid grid-rows-2">
        <div class=""></div>
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
            <!-- Scrollable area -->
            <Pane class="overflow-hidden">
                <!-- Important: overflow-hidden on Pane -->
                <div class="h-full overflow-y-auto p-4">
                    {#each packets as packet}
                        <div class="flex flex-row gap-20">
                            <p>
                                {packet.layers[packet.layers.length - 1].name}
                            </p>
                            {#each packet.layers[packet.layers.length - 1].fields as field}
                                <p>{field.value}</p>
                            {/each}
                        </div>
                    {/each}
                </div>
            </Pane>

            <PaneResizer class="w-full h-1 border-t-textd border-t" />
            <Pane defaultSize={25}></Pane>
        </PaneGroup>
    </div>
</div>
