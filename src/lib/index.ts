import { writable } from "svelte/store";

export const interface_index = writable<number>(0);

export async function get_interfaces() {
    let interfaces = await fetch(
        "http://127.0.0.1:8080/api/packets/get_interfaces",
        {
            method: "GET",
        },
    ).then((res) => res.json());
    return interfaces;
}
