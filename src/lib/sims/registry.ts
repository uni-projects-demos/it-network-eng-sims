interface SimDef {
  id: "tcp" | "udp" | "rip" | "ospf" | "flow" | "wsn";
  nameKey: string;
  groupKey: string;
  descriptionKey: string;
  status: "available" | "unavailable";
}

export const sims: SimDef[] = [
  {
    id: "tcp",
    nameKey: "tcpTitle",
    groupKey: "transport",
    descriptionKey: "tcpFileTransfer",
    status: "available",
  },
  {
    id: "udp",
    nameKey: "udpTitle",
    groupKey: "transport",
    descriptionKey: "udpFileTransfer",
    status: "available",
  },
  {
    id: "rip",
    nameKey: "ripTitle",
    groupKey: "network",
    descriptionKey: "ripRouting",
    status: "available",
  },
  {
    id: "ospf",
    nameKey: "ospfTitle",
    groupKey: "network",
    descriptionKey: "ospfRouting",
    status: "available",
  },
  {
    id: "wsn",
    nameKey: "wsnTitle",
    groupKey: "network",
    descriptionKey: "wsnSimulation",
    status: "available",
  },
  {
    id: "flow",
    nameKey: "flowTitle",
    groupKey: "network",
    descriptionKey: "networkFlowOptimisation",
    status: "available",
  },
];
