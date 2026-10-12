use super::INF;
use std::collections::BTreeMap;

const RES: u8 = 2;
const VRS: u8 = 2;
const ADDR_FAM_INET: u16 = 2;
const HDR_SIZE: usize = 4;
const ENTRY_SIZE: usize = 20;
const MAX_ENTRIES: usize = 25;

pub(super) fn encode_res(sender: u16, tbl: &BTreeMap<u16, u8>) -> Result<Vec<u8>, String> {
    if sender == 0 || sender > 64_000 {
        return Err("Invalid RIP sender router ID".into());
    }
    if tbl.is_empty() || tbl.len() > MAX_ENTRIES {
        return Err("RIP response must contain 1 to 25 entries".into());
    }
    let mut bytes = Vec::with_capacity(HDR_SIZE + ENTRY_SIZE * tbl.len());
    bytes.extend_from_slice(&[RES, VRS]);
    bytes.extend_from_slice(&sender.to_be_bytes());
    for (&dest, &metric) in tbl {
        if dest == 0 || dest > 64_000 || dest == sender || !(1..=INF).contains(&metric) {
            return Err("Invalid RIP destination or metric".into());
        }
        bytes.extend_from_slice(&ADDR_FAM_INET.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes()); // route tag
        bytes.extend_from_slice(&u32::from(dest).to_be_bytes()); // router ID in addr
        bytes.extend_from_slice(&0u32.to_be_bytes()); // subnet mask
        bytes.extend_from_slice(&0u32.to_be_bytes()); // next hop
        bytes.extend_from_slice(&u32::from(metric).to_be_bytes());
    }
    Ok(bytes)
}

pub(super) fn decode_res(bytes: &[u8], expected_sender: u16) -> Result<BTreeMap<u16, u8>, String> {
    if bytes.len() < HDR_SIZE + ENTRY_SIZE ||
        !(bytes.len() - HDR_SIZE).is_multiple_of(ENTRY_SIZE) ||
        (bytes.len() - HDR_SIZE) / ENTRY_SIZE > MAX_ENTRIES {
        return Err("Invalid RIP response length or entry count".into());
    }
    if bytes[0] != RES || bytes[1] != VRS {
        return Err("Invalid RIP command or version".into());
    }
    let sender = u16::from_be_bytes([bytes[2], bytes[3]]);
    if sender == 0 || sender > 64_000 || sender != expected_sender {
        return Err("Invalid or mismatched RIP sender ID".into());
    }
    let mut tbl = BTreeMap::new();
    for entry in bytes[HDR_SIZE..].as_chunks::<ENTRY_SIZE>().0 {
        let afi = u16::from_be_bytes([entry[0], entry[1]]);
        let tag = u16::from_be_bytes([entry[2], entry[3]]);
        let dest = u32::from_be_bytes(entry[4..8].try_into().expect("four bytes"));
        let mask = u32::from_be_bytes(entry[8..12].try_into().expect("four bytes"));
        let nxt_hop = u32::from_be_bytes(entry[12..16].try_into().expect("four bytes"));
        let metric = u32::from_be_bytes(entry[16..20].try_into().expect("four bytes"));
        if afi != ADDR_FAM_INET || tag != 0 || mask != 0 || nxt_hop != 0 ||
           !(1..=64_000).contains(&dest) || dest == u32::from(sender) ||
           !(1..=u32::from(INF)).contains(&metric) {
            return Err("Invalid RIP route entry fields".into());
        }
        if tbl.insert(dest as u16, metric as u8).is_some() {
            return Err("Duplicate RIP destination".into());
        }
    }
    Ok(tbl)
}
