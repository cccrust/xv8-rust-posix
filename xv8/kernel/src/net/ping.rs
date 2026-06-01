use crate::net::ipv4::Ipv4Proto;
use crate::net::{self, Be, Ipv4Addr, NetError};
use crate::net::NetworkHeader;
use crate::net::icmp::IcmpHeader;
use crate::param::NPING;
use crate::proc::{self, Channel};
use crate::spinlock::SpinLock;

const MAX_PING_QUEUE_DEPTH: usize = 8;
pub const MAX_PING_PAYLOAD: usize = 64;

struct PingReceiveEntry {
    src_ip: Ipv4Addr,
    data: [u8; MAX_PING_PAYLOAD],
    data_len: usize,
}

struct PingEntry {
    identifier: u16,
    next_seq: u16,
    receive_queue: [Option<PingReceiveEntry>; MAX_PING_QUEUE_DEPTH],
}

impl PingEntry {
    fn new(identifier: u16) -> Self {
        Self {
            identifier,
            next_seq: 1,
            receive_queue: [const { None }; MAX_PING_QUEUE_DEPTH],
        }
    }

    fn enqueue(&mut self, entry: PingReceiveEntry) -> Result<(), NetError> {
        let Some(slot) = self.receive_queue.iter_mut().find(|e| e.is_none()) else {
            err!(NetError::QueueFull)
        };
        *slot = Some(entry);
        Ok(())
    }

    fn dequeue(&mut self) -> Option<PingReceiveEntry> {
        self.receive_queue.iter_mut().find_map(|e| e.take())
    }
}

pub struct PingTable {
    entries: [Option<PingEntry>; NPING],
    next_identifier: u16,
}

static PING_TABLE: SpinLock<PingTable> = SpinLock::new(
    PingTable {
        entries: [const { None }; NPING],
        next_identifier: 1,
    },
    "ping",
);

impl PingTable {
    pub fn open() -> Result<usize, NetError> {
        let mut table = PING_TABLE.lock();

        let id = table.next_identifier;
        table.next_identifier = table.next_identifier.wrapping_add(1);
        if table.next_identifier == 0 {
            table.next_identifier = 1;
        }

        let Some(slot) = table.entries.iter().position(|e| e.is_none()) else {
            err!(NetError::TableFull)
        };

        table.entries[slot] = Some(PingEntry::new(id));
        Ok(slot)
    }

    pub fn close(socket_id: usize) {
        if socket_id < NPING {
            let mut table = PING_TABLE.lock();
            table.entries[socket_id] = None;
        }
    }

    pub fn send(socket_id: usize, dest_ip: Ipv4Addr, buf: &[u8]) -> Result<(), NetError> {
        if socket_id >= NPING {
            err!(NetError::BadSocket)
        }

        let (identifier, seq) = {
            let mut table = PING_TABLE.lock();
            let Some(entry) = table.entries[socket_id].as_mut() else {
                err!(NetError::BadSocket)
            };
            let seq = entry.next_seq;
            entry.next_seq = entry.next_seq.wrapping_add(1);
            (entry.identifier, seq)
        };

        let payload_len = buf.len().min(MAX_PING_PAYLOAD);

        let rest = ((identifier as u32) << 16) | (seq as u32);
        let icmp = IcmpHeader {
            r#type: 8,
            code: 0,
            sum: Be::new(0),
            rest: Be::new(rest),
        };
        let sum = net::internet_checksum(&[icmp.as_bytes(), &buf[..payload_len]]);
        let icmp = IcmpHeader {
            sum: Be::new(sum),
            ..icmp
        };

        net::transmit(dest_ip, Ipv4Proto::Icmp, &[icmp.as_bytes(), &buf[..payload_len]])
    }

    pub fn receive(
        socket_id: usize,
    ) -> Result<(Ipv4Addr, [u8; MAX_PING_PAYLOAD], usize), NetError> {
        if socket_id >= NPING {
            err!(NetError::BadSocket)
        }

        let mut table = PING_TABLE.lock();

        if table.entries[socket_id].is_none() {
            err!(NetError::BadSocket)
        }

        loop {
            if proc::current_proc().is_killed() {
                err!(NetError::Interrupted)
            }

            let Some(entry) = table.entries[socket_id].as_mut() else {
                err!(NetError::BadSocket)
            };

            if let Some(recv) = entry.dequeue() {
                return Ok((recv.src_ip, recv.data, recv.data_len));
            }

            table = proc::sleep(Channel::Buffer(entry as *const _ as usize), table);
        }
    }

    pub fn deliver(identifier: u16, src_ip: Ipv4Addr, data: &[u8]) -> Result<(), NetError> {
        let mut table = PING_TABLE.lock();

        let Some(entry) = table.entries.iter_mut().find_map(|e| {
            e.as_mut().filter(|s| s.identifier == identifier)
        }) else {
            return Ok(());
        };

        let data_len = data.len().min(MAX_PING_PAYLOAD);
        let mut data_buf = [0u8; MAX_PING_PAYLOAD];
        data_buf[..data_len].copy_from_slice(&data[..data_len]);

        let _ = entry.enqueue(PingReceiveEntry {
            src_ip,
            data: data_buf,
            data_len,
        });

        proc::wakeup(Channel::Buffer(entry as *const _ as usize));
        Ok(())
    }
}
