use std::{collections::HashMap, net::Ipv4Addr, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};

use actix_web::{Responder, web};
use pnet::{datalink::{self, NetworkInterface}, ipnetwork::Ipv4Network, packet::{arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket}, ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket}}, util::MacAddr};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::Packet;

use crate::packets;

pub fn get_interface_ip(interface: &NetworkInterface) -> Ipv4Addr {
    return interface.ips.iter().filter_map(|ip| {
        if let std::net::IpAddr::V4(ipv4) = ip.ip() {
            Some(ipv4)
        } else {
            None
        }
    }).next().expect("Interface has no IP addresses");
}

pub fn build_arp_request(
    source_mac: MacAddr,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> Vec<u8> {
    // Ethernet (14) + ARP (28) = 42 bytes
    let mut buffer = vec![0u8; 42];

    {
        let mut eth_pkt = MutableEthernetPacket::new(&mut buffer[..]).unwrap();
        eth_pkt.set_destination(MacAddr::broadcast());
        eth_pkt.set_source(source_mac);
        eth_pkt.set_ethertype(EtherTypes::Arp);

        // ARP payload starts after 14 bytes
        let mut arp_pkt = MutableArpPacket::new(&mut buffer[14..]).unwrap();
        arp_pkt.set_hardware_type(ArpHardwareTypes::Ethernet);
        arp_pkt.set_protocol_type(EtherTypes::Ipv4);
        arp_pkt.set_hw_addr_len(6);
        arp_pkt.set_proto_addr_len(4);
        arp_pkt.set_operation(ArpOperations::Request);
        arp_pkt.set_sender_hw_addr(source_mac);
        arp_pkt.set_sender_proto_addr(source_ip);
        arp_pkt.set_target_hw_addr(MacAddr::zero());
        arp_pkt.set_target_proto_addr(target_ip);
    }

    buffer
}

pub async fn discover_hosts(index: web::Path<u32>) -> impl Responder {
    let timeout = 3;
    // Get the interface with the provided name
    let interface = packets::get_interface(index.into_inner());
    if interface.is_none() {
        return web::Json(vec![(String::from("No interface found"), String::new(),  0.)]);
    }
    let interface = interface.unwrap();

    // Create the network address of the given interface
    let cidr: &Ipv4Network = &packets::get_network_addr(&interface);

    let source_mac = interface.mac.expect("Interface has no MAC address");
    let source_ip = get_interface_ip(&interface);
        // Open datalink channel
    let (mut tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Failed to open datalink channel: {}", e),
    };

    // Spawn receiver thread to collect replies
    let send_times = Arc::new(Mutex::new(HashMap::<Ipv4Addr, Instant>::new()));
    let send_times_clone = send_times.clone();
    let replies = Arc::new(Mutex::new(HashMap::<Ipv4Addr, (MacAddr, Duration)>::new()));
    let replies_clone = replies.clone();

    let receiver_handle = thread::spawn(move || {
        let ttl = Instant::now() + Duration::from_secs(timeout);
        while Instant::now() < ttl {
            match rx.next() {
                Ok(packet) => {
                    let end = Instant::now();
                    // Check Ethernet + ARP (42 bytes)
                    if packet.len() < 42 { continue; }
                    if let Some(eth) = pnet::packet::ethernet::EthernetPacket::new(packet) {
                        if eth.get_ethertype() != EtherTypes::Arp { continue; }

                        if let Some(arp) = pnet::packet::arp::ArpPacket::new(eth.payload()) {
                            if arp.get_operation() != ArpOperations::Reply { continue; }
                            let mut latency = Duration::from_secs(0);
                            let mut map = replies_clone.lock().unwrap();
                            let sip = arp.get_sender_proto_addr();
                            let smac = arp.get_sender_hw_addr(); // MAC addr
                            if let Some(sent_time) = send_times_clone.lock().unwrap().get(&sip) {
                                latency = end - *sent_time;
                            }
                            map.insert(sip, (smac, latency));
                        }
                    }
                }
                Err(_) => { }
            }
        }
    });

    for ip in cidr.iter().skip(1).take((cidr.size() - 2) as usize) {
        let packet = build_arp_request(source_mac, source_ip, ip);
        send_times.lock().unwrap().insert(ip, Instant::now());
        // send may fail depending on permissions
        if let Some(Err(e)) = tx.send_to(&packet, None) {
            eprintln!("Failed to send ARP to {}: {:?}", ip, e);
        }
    }

    receiver_handle.join().unwrap();
    let map = replies.lock().unwrap();
    let mut res = vec![];
    if !map.is_empty() {
        for (ip, (mac, latency)) in map.iter() {
            res.push((ip.to_string(), mac.to_string(), latency.as_secs_f32()));
        }
    }

    web::Json(res)
}