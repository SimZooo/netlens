<script lang="ts">
    let { data }: { data: number[] } = $props();
    let hex_vals = [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
    ];

    let hover_index = $state({ x: 0, y: 0 });
</script>

<div class="size-full p-2 overflow-y-auto grid grid-cols-2 gap-10">
    <div class="size-full">
        <table class="">
            <thead>
                <tr>
                    {#each hex_vals as h}
                        <th>{h}</th>
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#each Array(Math.ceil(data.length / 16)) as _, row_i}
                    <tr class="h-9">
                        {#each data.slice(row_i * 16, (row_i + 1) * 16) as byte, col_i}
                            <td
                                class="text-center hover:bg-foreground w-9"
                                class:hovered={hover_index.x == col_i &&
                                    hover_index.y == row_i}
                                onpointerenter={(hover_index = {
                                    x: col_i,
                                    y: row_i,
                                })}
                            >
                                {byte.toString(16).toUpperCase()}
                            </td>
                        {/each}
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
    <div class="size-full">
        <table class="">
            <thead>
                <tr>
                    {#each hex_vals as h}
                        <th>{h}</th>
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#each Array(Math.ceil(data.length / 16)) as _, row_i}
                    <tr class="h-9">
                        {#each data.slice(row_i * 16, (row_i + 1) * 16) as byte, col_i}
                            <td
                                class="text-center hover:bg-foreground w-9"
                                class:hovered={hover_index.x == col_i &&
                                    hover_index.y == row_i}
                                onpointerenter={(hover_index = {
                                    x: col_i,
                                    y: row_i,
                                })}
                            >
                                {byte >= 32 && byte <= 126
                                    ? String.fromCharCode(byte)
                                    : "."}
                            </td>
                        {/each}
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
</div>

<style>
    .hovered {
        background-color: #323539;
    }
</style>
