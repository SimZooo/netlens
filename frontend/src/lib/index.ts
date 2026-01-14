import type { TableHeader } from "svelte-infinitable/types";

export enum OsiLayer {
  Physical,
  DataLink,
  Network,
  Transport,
  Session,
  Presentation,
  Application,
}

export type Field = {
  name: string;
  value: string;
};

export type Layer = {
  protocol: string;
  name: string;
  osi_layer: OsiLayer;
  fields: Field[];
};

export type NetworkPacket = {
  src: string;
  dst: string;
  id: string;
  timestamp: number;
  raw: number[];
  layers: Layer[];
  length: number;
  last_protocol: string;
  time: string;
  info: string;
};

export function get_top_layer(packet: NetworkPacket): Layer {
  // Must have at least layer 1-3
  if (packet.layers.length === 0) {
    return {} as Layer;
  }

  return packet.layers[packet.layers.length - 1];
}

export let table_headers: TableHeader[] = [
  {
    label: "No.",
  },
  { label: "Time" },
  {
    label: "Source",
    filter: {
      type: "text",
      mode: "server",
      placeholder: "Filter by Source",
    },
  },
  {
    label: "Destination",
    filter: {
      type: "text",
      mode: "server",
      placeholder: "Filter by Destination",
    },
  },
  {
    label: "Protocol",
    filter: {
      type: "text",
      mode: "server",
      placeholder: "Filter by Protocol",
    },
  },
  { label: "Length" },
  { label: "Info" },
];
