use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::net::interface::{self, InterfaceId};
use crate::net::ipv4::Ipv4Proto;
use crate::net::route;
use crate::net::{self, Be, Ipv4Addr, NetError, NetworkHeader};
use crate::param::NTCP;
use crate::poll;
use crate::proc::{self, Channel};
use crate::spinlock::SpinLock;

pub const TCP_MAX_SEG: usize = 1460;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    LastAck,
    TimeWait,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TcpHeader {
    src_port: Be<u16>,
    dest_port: Be<u16>,
    seq_num: Be<u32>,
    ack_num: Be<u32>,
    off_flags: Be<u16>,
    window: Be<u16>,
    checksum: Be<u16>,
    urgent: Be<u16>,
}

const TCP_FIN: u8 = 1;
const TCP_SYN: u8 = 2;
const TCP_RST: u8 = 4;
const TCP_PSH: u8 = 8;
const TCP_ACK: u8 = 16;

unsafe impl NetworkHeader for TcpHeader {}

impl TcpHeader {
    pub const SIZE: usize = size_of::<Self>();

    pub fn new(src_port: u16, dest_port: u16, seq: u32, ack: u32, flags: u8, window: u16) -> Self {
        Self {
            src_port: Be::new(src_port),
            dest_port: Be::new(dest_port),
            seq_num: Be::new(seq),
            ack_num: Be::new(ack),
            off_flags: Be::new(((Self::SIZE as u16 / 4) << 12) | flags as u16),
            window: Be::new(window),
            checksum: Be::new(0),
            urgent: Be::new(0),
        }
    }

    fn flags(&self) -> u8 {
        (self.off_flags.get() & 0x3f) as u8
    }

    fn data_offset(&self) -> usize {
        ((self.off_flags.get() >> 12) * 4) as usize
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct TcpPseudoHeader {
    src_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
    zero: u8,
    proto: u8,
    tcp_len: Be<u16>,
}

unsafe impl NetworkHeader for TcpPseudoHeader {}

fn transmit_tcp(
    dest_ip: Ipv4Addr,
    dest_port: u16,
    src_port: u16,
    seq: u32,
    ack: u32,
    flags: u8,
    payload: &[u8],
) -> Result<(), NetError> {
    let route = route::best_route_for(dest_ip)?;
    let Some(interface) = interface::find_interface_by_id(route.interface_id) else { err!(NetError::NotConfigured) };
    let Some(src_ipv4) = interface.config.ipv4 else { err!(NetError::NotConfigured) };

    let mut header = TcpHeader::new(src_port, dest_port, seq, ack, flags, 65535);
    let pseudo = TcpPseudoHeader {
        src_ip: src_ipv4.addr,
        dest_ip,
        zero: 0,
        proto: Ipv4Proto::Tcp as u8,
        tcp_len: Be::new((TcpHeader::SIZE + payload.len()) as u16),
    };
    header.checksum = Be::new(net::internet_checksum(&[pseudo.as_bytes(), header.as_bytes(), payload]));

    net::transmit_with_route(route, interface, dest_ip, Ipv4Proto::Tcp, &[header.as_bytes(), payload])
}

#[derive(Debug)]
pub struct TcpConnection {
    state: TcpState,
    local_port: u16,
    remote_ip: Ipv4Addr,
    remote_port: u16,
    send_seq: u32,
    recv_seq: u32,
    send_buf: Vec<u8>,
    recv_buf: Vec<u8>,
    recv_ready: bool,
    backlog: Vec<usize>,
    pub nonblocking: bool,
    pub epoll_instances: Vec<usize>,
}

impl TcpConnection {
    fn new() -> Self {
        Self {
            state: TcpState::Closed,
            local_port: 0,
            remote_ip: Ipv4Addr::UNSPECIFIED,
            remote_port: 0,
            send_seq: 0,
            recv_seq: 0,
            send_buf: Vec::new(),
            recv_buf: Vec::new(),
            recv_ready: false,
            backlog: Vec::new(),
            nonblocking: false,
            epoll_instances: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TcpTable {
    pub entries: [Option<TcpConnection>; NTCP],
    next_ephemeral: u16,
}

const EPHEMERAL_PORT_START: u16 = 32768;

pub static TCP_TABLE: SpinLock<TcpTable> = SpinLock::new(
    TcpTable {
        entries: [const { None }; NTCP],
        next_ephemeral: EPHEMERAL_PORT_START,
    },
    "tcp_table",
);

impl TcpTable {
    fn alloc_port(&mut self) -> u16 {
        let port = self.next_ephemeral;
        self.next_ephemeral = self.next_ephemeral.wrapping_add(1);
        port
    }

    fn find_listener(&self, port: u16) -> Option<usize> {
        self.entries.iter().position(|e| {
            e.as_ref().is_some_and(|c| matches!(c.state, TcpState::Listen) && c.local_port == port)
        })
    }

    fn find_established(&self, remote_ip: Ipv4Addr, remote_port: u16, local_port: u16) -> Option<usize> {
        self.entries.iter().position(|e| {
            e.as_ref().is_some_and(|c| {
                c.remote_ip == remote_ip && c.remote_port == remote_port && c.local_port == local_port
                    && !matches!(c.state, TcpState::Closed | TcpState::Listen)
            })
        })
    }

    pub fn socket() -> Result<usize, NetError> {
        let mut table = TCP_TABLE.lock();
        let id = table.entries.iter().position(|e| e.is_none())
            .or_else(|| table.entries.iter().position(|e| matches!(e, Some(c) if c.state == TcpState::Closed)))
            .ok_or(NetError::TableFull)?;
        table.entries[id] = Some(TcpConnection::new());
        Ok(id)
    }

    pub fn bind(id: usize, port: u16) -> Result<(), NetError> {
        let mut table = TCP_TABLE.lock();
        if table.entries[id].is_none() { err!(NetError::BadSocket) }
        if table.entries[id].as_ref().unwrap().local_port != 0 { err!(NetError::AlreadyExists) }
        let bind_port = if port == 0 { table.alloc_port() } else { port };
        if let Some(_conflict) = table.find_listener(bind_port) { err!(NetError::PortInUse) }
        if let Some(entry) = table.entries[id].as_mut() {
            entry.local_port = bind_port;
            entry.state = TcpState::Listen;
        }
        Ok(())
    }

    pub fn listen(id: usize) -> Result<(), NetError> {
        let mut table = TCP_TABLE.lock();
        let Some(ref mut entry) = table.entries[id] else { err!(NetError::BadSocket) };
        if entry.local_port == 0 { err!(NetError::InvalidAddress) }
        entry.state = TcpState::Listen;
        Ok(())
    }

    pub fn connect(id: usize, remote_ip: Ipv4Addr, remote_port: u16) -> Result<(), NetError> {
        let (local_port, seq, is_nonblocking) = {
            let mut table = TCP_TABLE.lock();
            if table.entries[id].is_none() { err!(NetError::BadSocket) }
            let needs_port = table.entries[id].as_ref().unwrap().local_port == 0;
            if needs_port {
                let p = table.alloc_port();
                if let Some(entry) = table.entries[id].as_mut() {
                    entry.local_port = p;
                }
            }
            let entry = table.entries[id].as_mut().ok_or(NetError::BadSocket)?;
            entry.remote_ip = remote_ip;
            entry.remote_port = remote_port;
            entry.send_seq = 1000;
            entry.state = TcpState::SynSent;
            (entry.local_port, entry.send_seq, entry.nonblocking)
        };
        transmit_tcp(remote_ip, remote_port, local_port, seq, 0, TCP_SYN, &[])?;

        if is_nonblocking {
            return Err(NetError::ResourceUnavailable);
        }

        // Wait for handshake to complete
        loop {
            if proc::current_proc().is_killed() { err!(NetError::Interrupted) }
            let mut table = TCP_TABLE.lock();
            let entry = table.entries[id].as_mut().ok_or(NetError::BadSocket)?;
            if matches!(entry.state, TcpState::Established) {
                return Ok(());
            }
            if matches!(entry.state, TcpState::Closed) {
                err!(NetError::ConnectionRefused)
            }
            table = proc::sleep(Channel::Buffer(entry as *const _ as usize), table);
        }
    }

    pub fn accept(id: usize) -> Result<usize, NetError> {
        loop {
            if proc::current_proc().is_killed() { err!(NetError::Interrupted) }

            let backlog_id = {
                let mut table = TCP_TABLE.lock();
                let entry = table.entries[id].as_mut().ok_or(NetError::BadSocket)?;
                if !matches!(entry.state, TcpState::Listen) { err!(NetError::InvalidAddress) }
                if entry.backlog.is_empty() {
                    if entry.nonblocking {
                        err!(NetError::ResourceUnavailable)
                    }
                    let ptr = entry as *const _ as usize;
                    let _ = entry;
                    table = proc::sleep(Channel::Buffer(ptr), table);
                    continue;
                }
                Some(entry.backlog.remove(0))
            };
            if let Some(child_id) = backlog_id {
                return Ok(child_id);
            }
        }
    }

    pub fn close(id: usize) {
        let mut table = TCP_TABLE.lock();
        let Some(entry) = table.entries[id].as_mut() else { return };
        let state = entry.state;
        let local_port = entry.local_port;
        let remote_ip = entry.remote_ip;
        let remote_port = entry.remote_port;
        let seq = entry.send_seq;
        let ack = entry.recv_seq;

        match state {
            TcpState::Established => {
                entry.state = TcpState::FinWait1;
                drop(table);
                let _ = transmit_tcp(remote_ip, remote_port, local_port, seq, ack, TCP_FIN | TCP_ACK, &[]);
            }
            TcpState::CloseWait => {
                entry.state = TcpState::LastAck;
                drop(table);
                let _ = transmit_tcp(remote_ip, remote_port, local_port, seq, ack, TCP_FIN | TCP_ACK, &[]);
            }
            _ => {
                entry.state = TcpState::Closed;
            }
        }
    }

    pub fn send(id: usize, data: &[u8]) -> Result<usize, NetError> {
        let (seq, ack, lport, rip, rport) = {
            let table = TCP_TABLE.lock();
            let Some(ref entry) = table.entries[id] else { err!(NetError::BadSocket) };
            if !matches!(entry.state, TcpState::Established) { err!(NetError::NotConnected) }
            (entry.send_seq, entry.recv_seq, entry.local_port, entry.remote_ip, entry.remote_port)
        };
        let len = data.len().min(TCP_MAX_SEG);
        transmit_tcp(rip, rport, lport, seq, ack, TCP_PSH | TCP_ACK, &data[..len])?;

        let mut table = TCP_TABLE.lock();
        let Some(ref mut entry) = table.entries[id] else { err!(NetError::BadSocket) };
        entry.send_seq = entry.send_seq.wrapping_add(len as u32);
        Ok(len)
    }

    pub fn recv(id: usize, buf: &mut [u8]) -> Result<usize, NetError> {
        loop {
            if proc::current_proc().is_killed() { err!(NetError::Interrupted) }

            let mut table = TCP_TABLE.lock();
            let Some(ref mut entry) = table.entries[id] else { err!(NetError::BadSocket) };

            if entry.recv_ready && !entry.recv_buf.is_empty() {
                let len = entry.recv_buf.len().min(buf.len());
                buf[..len].copy_from_slice(&entry.recv_buf[..len]);
                entry.recv_buf.drain(..len);
                entry.recv_ready = !entry.recv_buf.is_empty();
                return Ok(len);
            }

            if matches!(entry.state, TcpState::Closed) { return Ok(0); }

            if entry.nonblocking {
                err!(NetError::ResourceUnavailable)
            }

            table = proc::sleep(Channel::Buffer(entry as *const _ as usize), table);
        }
    }
}

pub fn tcp_readiness(id: usize) -> (bool, bool) {
    let table = TCP_TABLE.lock();
    let Some(ref entry) = table.entries[id] else { return (false, false) };
    if matches!(entry.state, TcpState::Listen) {
        return (!entry.backlog.is_empty(), false);
    }
    let readable = entry.recv_ready && !entry.recv_buf.is_empty();
    let writable = matches!(entry.state, TcpState::Established);
    (readable, writable)
}

pub fn handle_tcp(src_ip: Ipv4Addr, dest_ip: Ipv4Addr, data: &[u8]) -> Result<(), NetError> {
    let Some((hdr, payload)) = TcpHeader::from_bytes_with_rest(data) else { err!(NetError::MalformedPacket) };

    let src_port = hdr.src_port.get();
    let dest_port = hdr.dest_port.get();
    let seq = hdr.seq_num.get();
    let ack = hdr.ack_num.get();
    let flags = hdr.flags();
    let has_syn = flags & TCP_SYN != 0;
    let has_ack = flags & TCP_ACK != 0;
    let has_fin = flags & TCP_FIN != 0;
    let has_rst = flags & TCP_RST != 0;

    // SYN (no ACK) → passive open, find listener
    if has_syn && !has_ack {
        let mut table = TCP_TABLE.lock();
        let Some(listener_id) = table.find_listener(dest_port) else { return Ok(()) };
        let child_id = table.entries.iter().position(|e| e.is_none())
            .or_else(|| table.entries.iter().position(|e| matches!(e, Some(c) if c.state == TcpState::Closed)))
            .ok_or(NetError::TableFull)?;
        let mut child = TcpConnection::new();
        child.state = TcpState::SynReceived;
        child.local_port = dest_port;
        child.remote_ip = src_ip;
        child.remote_port = src_port;
        child.recv_seq = seq.wrapping_add(1);
        child.send_seq = 2000;
        table.entries[child_id] = Some(child);
        let lport = dest_port;
        drop(table);
        let _ = transmit_tcp(src_ip, src_port, lport, 2000, seq.wrapping_add(1), TCP_SYN | TCP_ACK, &[]);
        return Ok(());
    }

    // SYN-ACK → client side handshake completion
    if has_syn && has_ack {
        let epoll_to_wake: Vec<(usize, u32)>;
        let mut table = TCP_TABLE.lock();
        for (id, entry) in table.entries.iter_mut().enumerate() {
            if let Some(c) = entry {
                if matches!(c.state, TcpState::SynSent) && c.remote_ip == src_ip && c.remote_port == src_port {
                    c.state = TcpState::Established;
                    c.send_seq = ack;
                    c.recv_seq = seq.wrapping_add(1);
                    let (lport, rip, rport, sseq, rseq) = (c.local_port, c.remote_ip, c.remote_port, c.send_seq, c.recv_seq);
                    epoll_to_wake = c.epoll_instances.iter().map(|&epfd| (epfd, poll::EPOLLOUT)).collect();
                    proc::wakeup(Channel::Buffer(c as *const _ as usize));
                    drop(table);
                    for (epfd, ev) in epoll_to_wake {
                        poll::epoll_notify_instances(epfd, ev);
                    }
                    let _ = transmit_tcp(rip, rport, lport, sseq, rseq, TCP_ACK, &[]);
                    return Ok(());
                }
            }
        }
        return Ok(());
    }

    // ACK of SYN-ACK → server side handshake completion
    if has_ack && !has_syn && !has_fin {
        let mut listener_epoll = Vec::new();
        let mut table = TCP_TABLE.lock();
        for child_id in 0..NTCP {
            let (state, local_port) = match &table.entries[child_id] {
                Some(c) if c.remote_ip == src_ip && c.remote_port == src_port => (c.state, c.local_port),
                _ => continue,
            };
            if !matches!(state, TcpState::SynReceived) { continue; }
            if let Some(c) = table.entries[child_id].as_mut() {
                c.state = TcpState::Established;
            }
            if let Some(listener) = table.find_listener(local_port) {
                if let Some(p) = table.entries[listener].as_mut() {
                    p.backlog.push(child_id);
                    listener_epoll = p.epoll_instances.clone();
                    proc::wakeup(Channel::Buffer(p as *const _ as usize));
                }
            }
        }
        drop(table);
        for epfd in listener_epoll {
            poll::epoll_notify_instances(epfd, poll::EPOLLIN);
        }
    }

    // Data delivery / FIN / RST to established connections
    if has_rst {
        let epoll_to_wake: Vec<(usize, u32)>;
        let mut table = TCP_TABLE.lock();
        if let Some(id) = table.find_established(src_ip, src_port, dest_port) {
            let c = table.entries[id].as_mut().unwrap();
            epoll_to_wake = c.epoll_instances.iter().map(|&epfd| (epfd, poll::EPOLLHUP | poll::EPOLLERR)).collect();
            c.state = TcpState::Closed;
            proc::wakeup(Channel::Buffer(c as *const _ as usize));
            drop(table);
            for (epfd, ev) in epoll_to_wake {
                poll::epoll_notify_instances(epfd, ev);
            }
        } else {
            drop(table);
        }
        return Ok(());
    }

    let mut epoll_to_wake: Vec<(usize, u32)> = Vec::new();
    let mut table = TCP_TABLE.lock();
    let Some(conn_id) = table.find_established(src_ip, src_port, dest_port) else { return Ok(()) };

    if has_fin {
        let conn = table.entries[conn_id].as_mut().unwrap();
        conn.state = TcpState::CloseWait;
        conn.recv_seq = seq.wrapping_add(1);
        conn.recv_ready = true;
        epoll_to_wake = conn.epoll_instances.iter().map(|&epfd| (epfd, poll::EPOLLIN | poll::EPOLLHUP)).collect();
        proc::wakeup(Channel::Buffer(conn as *const _ as usize));
        drop(table);
        for (epfd, ev) in epoll_to_wake {
            poll::epoll_notify_instances(epfd, ev);
        }
        return Ok(());
    }

    if !payload.is_empty() {
        let data_start = hdr.data_offset();
        let data = if data_start <= TcpHeader::SIZE {
            payload
        } else if data_start < TcpHeader::SIZE + payload.len() {
            &payload[(data_start - TcpHeader::SIZE)..]
        } else {
            &[]
        };

        let (lport, rip, rport, sseq, rseq) = {
            let conn = table.entries[conn_id].as_mut().unwrap();
            if !data.is_empty() {
                if conn.recv_buf.len() + data.len() <= 65536 {
                    conn.recv_buf.extend_from_slice(data);
                    conn.recv_seq = seq.wrapping_add(data.len() as u32);
                    conn.recv_ready = true;
                    epoll_to_wake = conn.epoll_instances.iter().map(|&epfd| (epfd, poll::EPOLLIN)).collect();
                    proc::wakeup(Channel::Buffer(conn as *const _ as usize));
                }
            }
            (conn.local_port, conn.remote_ip, conn.remote_port, conn.send_seq, conn.recv_seq)
        };
        drop(table);
        for (epfd, ev) in epoll_to_wake {
            poll::epoll_notify_instances(epfd, ev);
        }
        let _ = transmit_tcp(rip, rport, lport, sseq, rseq, TCP_ACK, &[]);
    }

    Ok(())
}