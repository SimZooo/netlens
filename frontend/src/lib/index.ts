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
  last_protocol: string;
  time: string;
};

export function get_top_layer(packet: NetworkPacket): Layer {
  // Must have at least layer 1-3
  if (packet.layers.length === 0) {
    return {} as Layer;
  }

  return packet.layers[packet.layers.length - 1];
}
