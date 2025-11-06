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

    let toggle = $state(false)
</script>

<div class="w-full h-fit text-[#B6BDBD] bg-[#1c2e3e] ps-2 rounded">
    <span aria-label="packet" role="button" tabindex=0 class="grid grid-rows-1 grid-cols-5" onclick={() => {toggle = !toggle; console.log(packet.eth_packet)}} onkeydown={(e) => {}} >
        <p>{get_packet_type()}</p>
        <p>{packet.eth_packet.src}</p>
        <p>{packet.eth_packet.dst}</p>
        {#if packet.tcp_packet}
            <p>{packet.tcp_packet.src}</p>
            <p>{packet.tcp_packet.dst}</p>
        {/if}

    </span>
    {#if toggle}
        <p class="wrap-break-word">{packet.eth_packet.packet_payload}</p>
    {/if}
</div>