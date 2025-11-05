<script>
    import { Chart } from "chart.js/auto";
    import { flushSync, onMount, tick } from "svelte";
    import Section from "../components/Section.svelte";
    import Modal from "../components/Modal.svelte";
    import PcapEl from "../components/PcapEl.svelte";

    let node_history = $state([]);
    let interface_history = $state([]);
    let node_width = 20;
    let gap = 8;
    let outer_div;
    let node_div;
    let interface_div;
    let gateway_avail = $state(0);
    let highlighted_host = $state();
    let pcap_toggle = $state(false);
    let pcap = $state([]);
    $effect(() => (gateway_avail = gateway_availability()));

    // Chartjs
    let ctx;
    let chart_canvas;
    let chart_object;

    let get_hosts_data = () => node_history.map((n) => n.number);

    let get_latency_data = () => {
        let data = [];

        for (const node of node_history) {
            for (const host of node.data) {
                data = [...data, host[2]];
            }
        }

        return data;
    };
    let hosts_data = get_hosts_data();
    let latency_data = get_latency_data();
    function chart(node, { hosts, latency }) {
        function setup_chart(_hosts, _latency) {
            chart_object = new Chart(node, {
                type: "line",
                data: {
                    labels: [""],
                    datasets: [
                        {
                            label: "Online hosts",
                            data: _hosts,
                            borderWidth: 1,
                            borderColor: "#EC7176",
                            backgroundColor: "#EC7176",
                        },
                        {
                            label: "Latency",
                            data: _latency,
                            borderWidth: 1,
                            borderColor: "#F4AB33",
                            backgroundColor: "#F4AB33",
                        },
                    ],
                },
                options: {
                    maintainAspectRatio: false,
                    fill: false,
                    plugins: {
                        legend: {
                            display: false,
                        },
                        tooltip: {
                            enabled: true,
                        },
                        title: {
                            display: false,
                        },
                    },
                    scales: {
                        x: {
                            display: false,
                        },
                        y: {
                            display: false,
                        },
                    },
                },
            });
        }
        setup_chart(hosts, latency);
        return {
            destroy() {
                chart_object.destroy();
            },
        };
    }

    // Update chart data
    $effect(() => {
        node_history.forEach((n) => n.number);
        if (!chart_object || !chart_object || node_history.length === 0) return;
        let node = node_history[node_history.length - 1];
        let latency =
            node.data.reduce((sum, host) => sum + host[2], 0) /
            node.data.length;

        chart_object.data.labels.push(node_history.length);

        chart_object.data.datasets[0].data = [
            ...chart_object.data.datasets[0].data,
            node.number,
        ];
        chart_object.data.datasets[1].data = [
            ...chart_object.data.datasets[1].data,
            latency,
        ];
        update_chart_canvas_width(node_history.length);
        chart_object.update();
    });

    async function scroll_to_right() {
        await tick();
        if (node_div) {
            node_div.scrollLeft = node_div.scrollWidth;
            interface_div.scrollLeft = interface_div.scrollWidth;
        }
    }

    function get_node_color(node) {
        if (node.number > 5) {
            return "#2ecc71";
        } else if (node.number > 3) {
            return "#f7c229";
        }
        return "#e74c3c";
    }

    function get_interface_node_color(node) {
        if (node.number >= 0) {
            return "#2ecc71";
        } else if (node.number >= 2) {
            return "#f7c229";
        }
        return "#e74c3c";
    }

    async function get_interfaces() {
        let interfaces = await fetch(
            "http://127.0.0.1:8080/api/packets/get_interfaces",
            {
                method: "GET",
            },
        ).then((res) => res.json());
        return interfaces;
    }

    async function get_hosts(index) {
        let hosts = await fetch(
            `http://127.0.0.1:8080/api/scan/discover_hosts/${index}`,
            {
                method: "GET",
            },
        ).then((res) => res.json());
        return hosts;
    }

    onMount(async () => {
        const ws = new WebSocket(
            "ws://127.0.0.1:8080/api/packets/ws/capture/0",
        );

        ws.onmessage = (event) => {
            if (pcap_toggle) {
                const packet = JSON.parse(event.data);
                pcap.push(packet);
            }
        };

        setInterval(async function () {
            let interfaces = await get_interfaces();
            interface_history = [
                ...interface_history,
                {
                    number: interfaces.length,
                    date: new Date(),
                    modal: false,
                    rect: undefined,
                    data: interfaces,
                },
            ];
            let hosts = await get_hosts(interfaces[0][1]);
            node_history = [
                ...node_history,
                {
                    number: hosts.length,
                    date: new Date(),
                    modal: false,
                    rect: undefined,
                    data: hosts,
                },
            ];
        }, 10 * 1000);

        await scroll_to_right();

        const resize_observer = new ResizeObserver(async () => {
            await scroll_to_right();
        });

        resize_observer.observe(outer_div);
    });

    const gateway_availability = () => {
        let availability = 0;
        for (let i = 0; i < node_history.length; i++) {
            let node = node_history[i];
            let data = node.data;
            let router = data.find((host) => host[0].endsWith(".1"));
            if (router) {
                availability += 1;
            }
        }
        return availability / node_history.length;
    };

    const get_gateway_color = () => {
        if (gateway_avail > 0.8) {
            return "#2ecc71";
        } else if (gateway_avail >= 0.5) {
            return "#f7c229";
        }
        return "#e74c3c";
    };

    function update_chart_canvas_width(data_length) {
        if (chart_canvas) {
        }
    }
</script>

<div class="w-full h-full flex flex-col gap-5">
    <div class="w-full grid grid-cols-2 gap-5">
        <div class="w-full h-full" bind:this={outer_div}>
            <Section title="Network Devices Status">
                <div
                    class="w-fit h-32 flex flex-col flex-wrap gap-2 overflow-x-auto p-2"
                    bind:this={node_div}
                >
                    {#each node_history as node}
                        <button
                            style="background-color: {get_node_color(node)}"
                            class="w-8 h-8 rounded"
                            onclick={(e) => {
                                node.modal = !node.modal;
                                node.rect = e.target.getBoundingClientRect();
                            }}
                        >
                            <Modal toggled={node.modal} rect={node.rect}>
                                <p class="text-sm">
                                    Time: {node.date.toLocaleDateString()}&nbsp;:&nbsp;{node.date.getHours()}.{node.date.getMinutes()}.{node.date.getSeconds()}
                                </p>
                                <p class="text-sm">Devices up: {node.number}</p>
                            </Modal>
                        </button>
                    {/each}
                </div>
            </Section>
        </div>
        <div class="w-full h-full">
            <Section title="Interface Status">
                <div
                    class="w-fit h-32 flex flex-col flex-wrap gap-2 overflow-x-auto p-2"
                    bind:this={interface_div}
                >
                    {#each interface_history as interface_node}
                        <button
                            style="background-color: {get_interface_node_color(
                                interface_node,
                            )}"
                            class="w-8 h-8 rounded"
                            onclick={(e) => {
                                interface_node.modal = !interface_node.modal;
                                interface_node.rect =
                                    e.target.getBoundingClientRect();
                            }}
                        >
                            <Modal
                                toggled={interface_node.modal}
                                rect={interface_node.rect}
                            >
                                <p class="text-sm">
                                    Time: {interface_node.date.toLocaleDateString()}&nbsp;:&nbsp;{interface_node.date.getHours()}.{interface_node.date.getMinutes()}.{interface_node.date.getSeconds()}
                                </p>
                                <p class="text-sm">
                                    Devices up: {interface_node.number}
                                </p>
                            </Modal>
                        </button>
                    {/each}
                </div>
            </Section>
        </div>
    </div>
    <div class="grid grid-cols-4 gap-5">
        <Section title="Network History" align="center">
            <div class="h-40 min-w-0 max-w-full overflow-x-scroll w-full">
                <canvas
                    use:chart={{ hosts: hosts_data, latency: latency_data }}
                    class=""
                    bind:this={chart_canvas}
                ></canvas>
            </div>
        </Section>
        <Section title="Reachable Hosts" align="center">
            <div class="h-40 overflow-y-auto">
                {#if node_history.length > 0}
                    {#each node_history[node_history.length - 1].data as host}
                        <p>{host[0]}&nbsp;:&nbsp;{host[1]}</p>
                    {/each}
                {/if}
            </div>
        </Section>
        <Section title="Default Gateway" align="center">
            <div
                class="flex flex-col w-50 h-40 justify-center text-center align-middle gap-1"
            >
                <h1 class="text-xl">Availability</h1>
                <p class="text-2xl" style="color: {get_gateway_color()}">
                    {Math.round(gateway_avail * 100)}%
                </p>
                {#if node_history.length > 0}
                    <p class="text-xl">
                        {node_history[node_history.length - 1].data.find((ip) =>
                            ip[0].endsWith(".1"),
                        )[0]}
                    </p>
                {/if}
            </div>
        </Section>
        <Section title="Highlighted Host" align="center">
            <div
                class="flex flex-col w-50 h-40 justify-center text-center align-middle gap-1"
            >
                <h1 class="text-xl">Availability</h1>
                {#if node_history.length > 0}
                    <p
                        class="text-2xl"
                        style="color: {node_history[
                            node_history.length - 1
                        ].data.find((ip) => ip[0] === highlighted_host)
                            ? '#2ecc71'
                            : '#e74c3c'}"
                    >
                        {node_history[node_history.length - 1].data.find(
                            (ip) => ip[0] === highlighted_host,
                        )
                            ? "Available"
                            : "Not Available"}
                    </p>
                {/if}
                <input
                    type="text"
                    placeholder="10.0.0.1"
                    class="text-xl text-center"
                    bind:value={highlighted_host}
                />
            </div>
        </Section>
    </div>
    <div>
        <Section title="Packet Analysis">
            <div class="h-180 w-full p-2">
                <button
                    class="flex flex-row h-8 rounded m-4 place-items-center border-2 p-0 relative overflow-hidden"
                    onclick={() => (pcap_toggle = !pcap_toggle)}
                >
                    <div class="relative flex w-26 h-7">
                        <span id="pcap-on" class="w-13 m-0 p-0 h-7">ON</span>
                        <span id="pcap-off" class="w-13 m-0 p-0 h-7">OFF</span>
                        <span
                            class="w-13 m-0 p-0 h-7 absolute bg-[#B6BDBD] transition"
                            style={pcap_toggle ? "transform:translateX(100%)" : ""}
                        ></span>
                    </div>
                </button>
                <div class="flex flex-col gap-2 w-full">
                    <span class="grid grid-rows-1 grid-cols-5 w-full h-fit ps-2 rounded">
                        <p>Type</p>
                        <p>Eth Src</p>
                        <p>Eth Dst</p>
                        <p>Port Src</p>
                        <p>Port Dst</p>
                    </span>
                    {#each pcap as packet}
                        <PcapEl packet={packet} />
                    {/each}
                </div>
            </div>
        </Section>
    </div>
</div>

<style>
    *::-webkit-scrollbar {
        display: none;
    }
</style>
