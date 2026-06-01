use crate::net::ipv4::Ipv4Proto;
use crate::net::{self, Be, Ipv4Addr, NetError};
use crate::net::NetworkHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IcmpType {
    EchoReply = 0,
    EchoRequest = 8,
    Unknown = u8::MAX,
}

impl From<u8> for IcmpType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::EchoReply,
            8 => Self::EchoRequest,
            _ => Self::Unknown,
        }
    }
}

/// ICMP Header
/// https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol#Header
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct IcmpHeader {
    /// Type
    pub(crate) r#type: u8,
    /// Code
    pub(crate) code: u8,
    /// Checksum
    pub(crate) sum: Be<u16>,
    /// Rest of Header (varies by type/code)
    pub(crate) rest: Be<u32>,
}

impl IcmpHeader {
    fn r#type(&self) -> IcmpType {
        IcmpType::from(self.r#type)
    }

    fn new_echo_reply(request: &IcmpHeader) -> Self {
        Self {
            r#type: IcmpType::EchoReply as u8,
            code: 0,
            sum: Be::new(0), // to be filled later
            rest: request.rest,
        }
    }

    fn calculate_checksum(&self, data: &[u8]) -> u16 {
        let mut header = *self;
        header.sum = Be::new(0);
        net::internet_checksum(&[header.as_bytes(), data])
    }

    fn add_checksum(mut self, data: &[u8]) -> Self {
        self.sum = Be::new(self.calculate_checksum(data));
        self
    }
}

unsafe impl NetworkHeader for IcmpHeader {}

/// Sends an ICMP Echo Request.
pub fn send_echo_request(
    dest_ip: Ipv4Addr,
    identifier: u16,
    seq: u16,
    data: &[u8],
) -> Result<(), NetError> {
    let rest = ((identifier as u32) << 16) | (seq as u32);
    let icmp = IcmpHeader {
        r#type: IcmpType::EchoRequest as u8,
        code: 0,
        sum: Be::new(0),
        rest: Be::new(rest),
    };
    let icmp = icmp.add_checksum(data);
    log!(net::transmit(
        dest_ip,
        Ipv4Proto::Icmp,
        &[icmp.as_bytes(), data]
    ))
}

/// Sends an ICMP Echo Reply in response to the given ICMP Echo Request.
pub fn echo_reply(
    dest_ip: Ipv4Addr,
    req_icmp: &IcmpHeader,
    req_data: &[u8],
) -> Result<(), NetError> {
    let icmp = IcmpHeader::new_echo_reply(req_icmp).add_checksum(req_data);

    log!(net::transmit(
        dest_ip,
        Ipv4Proto::Icmp,
        &[icmp.as_bytes(), req_data]
    ))
}

/// Handles an incoming ICMP packet.
/// If it's an Echo Request, sends back an Echo Reply.
/// Otherwise, ignores it.
pub fn handle_icmp(dest_ip: Ipv4Addr, req_data: &[u8]) -> Result<(), NetError> {
    if let Some((req_icmp, req_data)) = IcmpHeader::from_bytes_with_rest(req_data) {
        match req_icmp.r#type() {
            IcmpType::EchoRequest => {
                log!(echo_reply(dest_ip, req_icmp, req_data))
            }
             IcmpType::EchoReply => {
                 let identifier = (req_icmp.rest.get() >> 16) as u16;
                 log!(crate::net::ping::PingTable::deliver(identifier, dest_ip, req_data))
             }
            IcmpType::Unknown => Ok(()),
        }
    } else {
        err!(NetError::MalformedPacket)
    }
}
