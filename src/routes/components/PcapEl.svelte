<script>
    let { packet } = $props();
    let get_packet_type = () => {
        if (packet.tcp_packet) {
            return "TCP"
        } else if (packet.ip_packet) {
            return "IP"
        } else {
            return "ETH"
        }
    }
</script>

<span class="grid grid-rows-1 grid-cols-5 w-full h-fit text-[#0C141B] bg-[#B6BDBD] ps-2 rounded">
    <p>{get_packet_type()}</p>
    <p>{packet.eth_packet.src}</p>
    <p>{packet.eth_packet.dst}</p>
    {#if packet.tcp_packet}
        <p>{packet.tcp_packet.src}</p>
        <p>{packet.tcp_packet.dst}</p>
    {/if}
</span>