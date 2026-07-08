use std::{cmp::Ordering, collections::HashMap};

use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo,
};
use sysinfo::System;

use crate::model::{SocketRow, SortMode};

pub(crate) fn socket_rows(system: &System, sort: SortMode) -> Vec<SocketRow> {
    let names = process_names(system);
    let flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let protocols = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let Ok(sockets) = get_sockets_info(flags, protocols) else {
        return Vec::new();
    };
    let mut rows: Vec<_> = sockets
        .into_iter()
        .map(|socket| socket_row(socket.protocol_socket_info, socket.associated_pids, &names))
        .collect();
    rows.sort_by(|left, right| compare_socket(left, right, sort));

    rows
}

fn compare_socket(left: &SocketRow, right: &SocketRow, sort: SortMode) -> Ordering {
    match sort {
        SortMode::Pid => left.pid.cmp(&right.pid),
        SortMode::State => left.state.cmp(&right.state).then_with(|| left.pid.cmp(&right.pid)),
        SortMode::Name => left.process.to_lowercase().cmp(&right.process.to_lowercase()),
        _ => left
            .protocol
            .cmp(&right.protocol)
            .then_with(|| left.local.cmp(&right.local))
            .then_with(|| left.remote.cmp(&right.remote)),
    }
}
fn process_names(system: &System) -> HashMap<String, String> {
    system
        .processes()
        .iter()
        .map(|(pid, process)| (pid.to_string(), process.name().to_string_lossy().into_owned()))
        .collect()
}

fn socket_row(
    info: ProtocolSocketInfo,
    pids: Vec<u32>,
    names: &HashMap<String, String>,
) -> SocketRow {
    let pid = pids.first().map(u32::to_string).unwrap_or_else(|| "-".to_string());
    let process = names.get(pid.as_str()).cloned().unwrap_or_else(|| "-".to_string());
    match info {
        ProtocolSocketInfo::Tcp(tcp) => SocketRow {
            protocol: "TCP".to_string(),
            local: endpoint(tcp.local_addr.to_string(), tcp.local_port),
            remote: endpoint(tcp.remote_addr.to_string(), tcp.remote_port),
            state: format!("{:?}", tcp.state),
            pid,
            process,
        },
        ProtocolSocketInfo::Udp(udp) => SocketRow {
            protocol: "UDP".to_string(),
            local: endpoint(udp.local_addr.to_string(), udp.local_port),
            remote: "-".to_string(),
            state: "-".to_string(),
            pid,
            process,
        },
    }
}

fn endpoint(address: String, port: u16) -> String {
    if address.contains(':') {
        format!("[{address}]:{port}")
    } else {
        format!("{address}:{port}")
    }
}