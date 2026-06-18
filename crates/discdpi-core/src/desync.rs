//! TCP TLS desync strategies (fake, multisplit).

use crate::packet::{build_ipv4_tcp_packet, Ipv4TcpInfo};
use crate::strategy::{DesyncMethod, DesyncParams, Stage};
use crate::tls;

/// Decides whether a TLS flow should receive desync treatment.
pub trait DesyncTargetFilter {
    fn matches_tls(&self, dst_port: u16, sni: Option<&str>, is_client_hello: bool) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessOutcome {
    Passthrough,
    Modified(Vec<Vec<u8>>),
}

#[derive(Debug, Clone)]
pub struct DesyncEngine {
    methods: Vec<DesyncMethod>,
    params: DesyncParams,
}

impl DesyncEngine {
    pub fn from_tcp_stage(stage: &Stage) -> Self {
        Self {
            methods: stage.desync.clone(),
            params: stage.desync_params.clone().unwrap_or_default(),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.methods.is_empty()
    }

    pub fn inactive() -> Self {
        Self {
            methods: Vec::new(),
            params: DesyncParams::default(),
        }
    }

    pub fn process(&self, packet: &[u8], filter: &impl DesyncTargetFilter) -> ProcessOutcome {
        let Some(info) = Ipv4TcpInfo::parse(packet) else {
            return ProcessOutcome::Passthrough;
        };

        let payload = info.payload();
        if payload.is_empty() || !info.has_psh() {
            return ProcessOutcome::Passthrough;
        }

        if !should_desync(&info, payload, filter) {
            return ProcessOutcome::Passthrough;
        }

        let mut out = Vec::new();

        if self.methods.contains(&DesyncMethod::Fake) {
            for _ in 0..self.params.fake_repeats {
                if let Some(fake) = fake_packet(packet, self.params.fake_ttl) {
                    out.push(fake);
                }
            }
        }

        if self.methods.contains(&DesyncMethod::Multisplit) {
            match multisplit(&info, payload, self.params.split_pos) {
                Some(parts) => out.extend(parts),
                None => out.push(packet.to_vec()),
            }
        } else {
            out.push(packet.to_vec());
        }

        if out.is_empty() {
            ProcessOutcome::Passthrough
        } else {
            ProcessOutcome::Modified(out)
        }
    }
}

fn should_desync(info: &Ipv4TcpInfo<'_>, payload: &[u8], filter: &impl DesyncTargetFilter) -> bool {
    let sni = tls::parse_client_hello_sni(payload);
    filter.matches_tls(
        info.dst_port(),
        sni.as_deref(),
        tls::is_tls_client_hello(payload),
    )
}

fn fake_packet(packet: &[u8], ttl: u8) -> Option<Vec<u8>> {
    let info = Ipv4TcpInfo::parse(packet)?;
    Some(build_ipv4_tcp_packet(
        info.src_ip(),
        info.dst_ip(),
        ttl,
        info.src_port(),
        info.dst_port(),
        info.sequence(),
        info.acknowledgment(),
        info.has_psh(),
        info.has_ack(),
        info.payload(),
    ))
}

fn multisplit(info: &Ipv4TcpInfo<'_>, payload: &[u8], split_pos: usize) -> Option<Vec<Vec<u8>>> {
    if !tls::is_tls_client_hello(payload) || split_pos == 0 || split_pos >= payload.len() {
        return None;
    }

    let (first, second) = payload.split_at(split_pos);
    let seq = info.sequence();

    let first_pkt = build_ipv4_tcp_packet(
        info.src_ip(),
        info.dst_ip(),
        info.ttl(),
        info.src_port(),
        info.dst_port(),
        seq,
        info.acknowledgment(),
        true,
        info.has_ack(),
        first,
    );

    let second_pkt = build_ipv4_tcp_packet(
        info.src_ip(),
        info.dst_ip(),
        info.ttl(),
        info.src_port(),
        info.dst_port(),
        seq.wrapping_add(split_pos as u32),
        info.acknowledgment(),
        true,
        info.has_ack(),
        second,
    );

    Some(vec![first_pkt, second_pkt])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{DesyncMethod, DesyncParams, Profile, Stage};

    struct TestFilter;

    impl DesyncTargetFilter for TestFilter {
        fn matches_tls(&self, dst_port: u16, sni: Option<&str>, is_client_hello: bool) -> bool {
            dst_port == 443 && sni == Some("discord.com") && is_client_hello
        }
    }

    fn test_engine() -> DesyncEngine {
        DesyncEngine::from_tcp_stage(&Stage {
            protocol: "tcp".to_string(),
            ports: vec!["443".to_string()],
            desync: vec![DesyncMethod::Multisplit, DesyncMethod::Fake],
            desync_params: Some(DesyncParams {
                split_pos: 1,
                fake_ttl: 2,
                fake_repeats: 1,
            }),
        })
    }

    #[test]
    fn profile_loads_desync_params() {
        let profile = Profile::from_toml(
            r#"
            [name]
            id = "default"

            [[stages]]
            protocol = "tcp"
            ports = ["443"]
            desync = ["multisplit"]

            [stages.desync_params]
            split_pos = 3
            fake_ttl = 4
            fake_repeats = 2
            "#,
        )
        .expect("profile");

        let params = profile.stages[0].desync_params.as_ref().expect("params");
        assert_eq!(params.split_pos, 3);
        assert_eq!(params.fake_repeats, 2);
    }

    #[test]
    fn non_tls_packet_is_passthrough() {
        let packet = build_ipv4_tcp_packet(
            [192, 168, 1, 10],
            [93, 184, 216, 34],
            64,
            50_000,
            443,
            1,
            0,
            true,
            false,
            b"GET /",
        );
        let engine = test_engine();
        assert_eq!(
            engine.process(&packet, &TestFilter),
            ProcessOutcome::Passthrough
        );
    }

    /// Minimal TLS ClientHello with SNI `discord.com` (enough for our parser).
    fn sample_discord_client_hello() -> Vec<u8> {
        vec![
            0x16, 0x03, 0x01, 0x00, 0x43, // TLS record
            0x01, 0x00, 0x00, 0x3f, // ClientHello, len 63
            0x03, 0x03, // TLS 1.2
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // random (32 bytes)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, // session id length
            0x00, 0x02, 0x00, 0x2f, // one cipher suite
            0x01, 0x00, // compression
            0x00, 0x14, // extensions length
            0x00, 0x00, 0x00, 0x10, // SNI extension
            0x00, 0x0e, // server_name list length
            0x00, // host_name type
            0x00, 0x0b, // host_name length
            b'd', b'i', b's', b'c', b'o', b'r', b'd', b'.', b'c', b'o', b'm',
        ]
    }

    #[test]
    fn discord_client_hello_triggers_modified_desync() {
        let tls_payload = sample_discord_client_hello();
        assert_eq!(
            tls::parse_client_hello_sni(&tls_payload).as_deref(),
            Some("discord.com")
        );

        let packet = build_ipv4_tcp_packet(
            [192, 168, 1, 10],
            [162, 159, 128, 234], // discord.com A record (example)
            64,
            52_341,
            443,
            1_000,
            0,
            true,
            false,
            &tls_payload,
        );

        let engine = test_engine();
        match engine.process(&packet, &TestFilter) {
            ProcessOutcome::Modified(packets) => {
                // 1 fake + 2 multisplit segments
                assert_eq!(packets.len(), 3);

                let fake = Ipv4TcpInfo::parse(&packets[0]).expect("fake");
                assert_eq!(fake.ttl(), 2);
                assert_eq!(fake.payload(), &tls_payload);

                let first = Ipv4TcpInfo::parse(&packets[1]).expect("first");
                let second = Ipv4TcpInfo::parse(&packets[2]).expect("second");
                assert_eq!(first.payload(), &tls_payload[..1]);
                assert_eq!(second.payload(), &tls_payload[1..]);
                assert_eq!(
                    second.sequence(),
                    first.sequence().wrapping_add(first.payload().len() as u32)
                );
            }
            ProcessOutcome::Passthrough => panic!("expected Modified outcome"),
        }
    }
}
