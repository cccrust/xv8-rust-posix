use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;

use crate::net::interface::{self, InterfaceConfig, InterfaceId, NetDevice};
use crate::net::{self, MacAddr, NetError};
use crate::rng;
use crate::spinlock::SpinLock;
use crate::sync::OnceLock;
use crate::vm::VA;

/// A single end of a veth pair, implementing NetDevice.
/// Transmit() enqueues the packet to the peer's incoming queue.
struct VethEndpoint {
    peer_id: OnceLock<InterfaceId>,
    own_id: OnceLock<InterfaceId>,
}

impl NetDevice for VethEndpoint {
    fn transmit(&self, packet: &[u8]) -> Result<(), NetError> {
        let peer = self.peer_id.get().ok_or(NetError::NotConfigured)?;
        net::enqueue_incoming(*peer, Box::from(packet))
    }

    fn needs_arp(&self) -> bool {
        false
    }

    fn needs_dhcp(&self) -> bool {
        false
    }
}

fn random_mac() -> MacAddr {
    let mut buf = [0u8; 6];
    rng::rand_bytes(&mut buf);
    // Set local/admin bit and clear multicast bit
    buf[0] = (buf[0] & 0xfe) | 0x02;
    MacAddr(buf)
}

/// Create a veth pair with the given names.
/// Returns the interface IDs of both ends.
pub fn create_pair(name1: &str, name2: &str) -> Result<(InterfaceId, InterfaceId), NetError> {
    let dev1 = Arc::new(VethEndpoint {
        peer_id: OnceLock::new(),
        own_id: OnceLock::new(),
    });
    let dev2 = Arc::new(VethEndpoint {
        peer_id: OnceLock::new(),
        own_id: OnceLock::new(),
    });

    let id1 = interface::register_interface(
        InterfaceConfig {
            name: String::from(name1),
            mac: random_mac(),
            ipv4: None,
            is_up: true,
        },
        dev1.clone(),
    );
    let id2 = interface::register_interface(
        InterfaceConfig {
            name: String::from(name2),
            mac: random_mac(),
            ipv4: None,
            is_up: true,
        },
        dev2.clone(),
    );

    // Wire up peer references
    dev1.peer_id.initialize(|| Ok::<_, ()>(id2));
    dev2.peer_id.initialize(|| Ok::<_, ()>(id1));
    dev1.own_id.initialize(|| Ok::<_, ()>(id1));
    dev2.own_id.initialize(|| Ok::<_, ()>(id2));

    Ok((id1, id2))
}

/// Ioctl handler: creates a veth pair.
/// arg points to a struct with two 16-byte name fields (32 bytes total).
pub fn ioctl_create_veth(arg: VA) -> Result<usize, crate::syscall::SysError> {
    let mut buf = [0u8; 32];
    if crate::proc::copy_from_user(arg, &mut buf).is_err() {
        return Err(crate::syscall::SysError::BadAddress);
    }

    let name1_bytes = &buf[..16];
    let name2_bytes = &buf[16..32];

    let name1_len = name1_bytes.iter().position(|&b| b == 0).unwrap_or(16);
    let name2_len = name2_bytes.iter().position(|&b| b == 0).unwrap_or(16);

    let name1 = core::str::from_utf8(&name1_bytes[..name1_len])
        .map_err(|_| crate::syscall::SysError::InvalidArgument)?;
    let name2 = core::str::from_utf8(&name2_bytes[..name2_len])
        .map_err(|_| crate::syscall::SysError::InvalidArgument)?;

    if name1.is_empty() || name2.is_empty() {
        return Err(crate::syscall::SysError::InvalidArgument);
    }

    let (_id1, _id2) = create_pair(name1, name2)
        .map_err(|_| crate::syscall::SysError::InvalidArgument)?;

    Ok(0)
}
