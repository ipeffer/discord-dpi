//! IPv4/TCP packet helpers for capture-layer desync.

use etherparse::PacketBuilder;

#[derive(Debug, Clone, Copy)]
pub struct Ipv4TcpInfo<'a> {
    pub raw: &'a [u8],
    ip_header_len: usize,
    tcp_header_len: usize,
}

impl<'a> Ipv4TcpInfo<'a> {
    pub fn parse(packet: &'a [u8]) -> Option<Self> {
        if packet.len() < 40 {
            return None;
        }
        if packet[0] >> 4 != 4 {
            return None;
        }

        let ip_header_len = (packet[0] & 0x0f) as usize * 4;
        if ip_header_len < 20 || packet.len() < ip_header_len + 20 || packet[9] != 6 {
            return None;
        }

        let tcp_start = ip_header_len;
        let tcp_header_len = ((packet[tcp_start + 12] >> 4) as usize) * 4;
        if tcp_header_len < 20 || packet.len() < tcp_start + tcp_header_len {
            return None;
        }

        Some(Self {
            raw: packet,
            ip_header_len,
            tcp_header_len,
        })
    }

    pub fn src_ip(&self) -> [u8; 4] {
        self.raw[12..16].try_into().expect("ipv4 src")
    }

    pub fn dst_ip(&self) -> [u8; 4] {
        self.raw[16..20].try_into().expect("ipv4 dst")
    }

    pub fn ttl(&self) -> u8 {
        self.raw[8]
    }

    pub fn src_port(&self) -> u16 {
        let off = self.ip_header_len;
        u16::from_be_bytes([self.raw[off], self.raw[off + 1]])
    }

    pub fn dst_port(&self) -> u16 {
        let off = self.ip_header_len + 2;
        u16::from_be_bytes([self.raw[off], self.raw[off + 1]])
    }

    pub fn sequence(&self) -> u32 {
        let off = self.ip_header_len + 4;
        u32::from_be_bytes([
            self.raw[off],
            self.raw[off + 1],
            self.raw[off + 2],
            self.raw[off + 3],
        ])
    }

    pub fn acknowledgment(&self) -> u32 {
        let off = self.ip_header_len + 8;
        u32::from_be_bytes([
            self.raw[off],
            self.raw[off + 1],
            self.raw[off + 2],
            self.raw[off + 3],
        ])
    }

    pub fn flags(&self) -> u8 {
        self.raw[self.ip_header_len + 13]
    }

    pub fn has_psh(&self) -> bool {
        self.flags() & 0x08 != 0
    }

    pub fn has_ack(&self) -> bool {
        self.flags() & 0x10 != 0
    }

    pub fn payload(&self) -> &[u8] {
        &self.raw[self.ip_header_len + self.tcp_header_len..]
    }
}

pub fn build_ipv4_tcp_packet(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    ttl: u8,
    src_port: u16,
    dst_port: u16,
    sequence: u32,
    acknowledgment: u32,
    psh: bool,
    ack: bool,
    payload: &[u8],
) -> Vec<u8> {
    let mut builder = PacketBuilder::ipv4(src_ip, dst_ip, ttl).tcp(
        src_port,
        dst_port,
        sequence,
        65_535,
    );

    if ack {
        builder = builder.ack(acknowledgment);
    }
    if psh {
        builder = builder.psh();
    }

    let mut packet = Vec::with_capacity(builder.size(payload.len()));
    builder
        .write(&mut packet, payload)
        .expect("packet builder write");
    packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_builder_and_parser() {
        let payload = b"hello";
        let built = build_ipv4_tcp_packet(
            [10, 0, 0, 1],
            [1, 1, 1, 1],
            64,
            44_000,
            443,
            100,
            200,
            true,
            true,
            payload,
        );

        let info = Ipv4TcpInfo::parse(&built).expect("parse");
        assert_eq!(info.payload(), payload);
        assert_eq!(info.dst_port(), 443);
    }
}
