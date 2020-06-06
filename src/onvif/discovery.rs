use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::soap::headers::Probe;
use crate::soap::Envelope;
use crate::soap::SoapClient;

const MULTICAST_ADDR: &str = "239.255.255.250:3702";

const DEVICE_TYPES: [&str; 3] = ["NetworkVideoTransmitter", "Device", "NetworkVideoDisplay"];
const READ_TIMEOUT: u64 = 300;
const RETRY_TIMES: usize = 3;

#[derive(Deserialize)]
struct EndpointReference {
    #[serde(rename = "Address")]
    address: String,
}

#[derive(Deserialize)]
struct RawProbeMatch {
    #[serde(rename = "XAddrs")]
    xaddrs: String,
    #[serde(rename = "Scopes")]
    scopes: String,
    #[serde(rename = "EndpointReference")]
    endpoint_reference: EndpointReference,
}

#[derive(Deserialize)]
struct ProbeMatchesContainer {
    #[serde(rename = "ProbeMatch")]
    probe_matches: Vec<RawProbeMatch>,
}

#[derive(Deserialize)]
struct DiscoveryBody {
    #[serde(rename = "ProbeMatches")]
    probe_matches_container: ProbeMatchesContainer,
}

#[derive(Hash, Eq, PartialEq, Debug, Serialize)]
pub struct ProbeMatch {
    name: String,
    #[serde(skip)]
    id: String,
    xaddrs: Vec<String>,
}

impl From<RawProbeMatch> for ProbeMatch {
    fn from(raw_probe_match: RawProbeMatch) -> Self {
        let id = raw_probe_match.endpoint_reference.address[9..].to_string();

        let name = raw_probe_match
            .scopes
            .split(' ')
            .find(|scope| scope.starts_with("onvif://www.onvif.org/name"))
            .unwrap()
            .split('/')
            .last()
            .unwrap()
            .to_string();

        let xaddrs: Vec<String> = raw_probe_match
            .xaddrs
            .split(' ')
            .map(|xaddr| xaddr.to_string())
            .collect();

        Self { name, id, xaddrs }
    }
}

pub fn discovery() -> Vec<ProbeMatch> {
    let socket = create_socket();

    multicast_probe_messages(&socket);

    let responses = recv_all_responses(&socket);

    parse_responses(responses)
}

fn create_socket() -> UdpSocket {
    let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
    let socket = UdpSocket::bind(free_socket_addr).unwrap();

    let timeout = Duration::from_millis(READ_TIMEOUT);
    socket.set_read_timeout(Some(timeout)).unwrap();

    socket
}

fn multicast_probe_messages(socket: &UdpSocket) {
    let multicast_addr: SocketAddr = MULTICAST_ADDR.parse().unwrap();

    let messages = DEVICE_TYPES
        .iter()
        .map(|device_type| {
            let client = SoapClient {
                header: Probe::new(Uuid::new_v4()),
            };

            client.build(|writer| {
                writer
                    .new_event("d:Probe")
                    .ns("d", "http://schemas.xmlsoap.org/ws/2005/04/discovery")
                    .write();

                writer
                    .new_event("d:Types")
                    .ns("dp0", "http://www.onvif.org/ver10/network/wsdl")
                    .content(&format!("dp0:{}", device_type))
                    .end();

                writer.end_event(); // Probe
            })
        })
        .collect::<Vec<String>>();

    for message in messages {
        for _ in 0..RETRY_TIMES {
            socket.send_to(message.as_bytes(), multicast_addr).unwrap();
        }
    }
}

fn recv_all_responses(socket: &UdpSocket) -> Vec<String> {
    let mut responses = Vec::new();
    loop {
        let mut buf = [0; 65_535];

        // TODO: Probably it can be async
        match socket.recv(&mut buf) {
            Ok(amt) => {
                let string = String::from_utf8(buf[..amt].to_vec()).unwrap();

                responses.push(string);
            }
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => break,
                _ => panic!(err),
            },
        }
    }

    responses
}

fn parse_responses(responses: Vec<String>) -> Vec<ProbeMatch> {
    let parsed_responses = responses
        .into_iter()
        .map(|response| serde_xml_rs::from_str(&response).unwrap())
        .collect::<Vec<Envelope<DiscoveryBody>>>();

    let unique_probe_matches = parsed_responses
        .into_iter()
        .flat_map(|response| response.body.probe_matches_container.probe_matches)
        .map(ProbeMatch::from)
        .collect::<HashSet<ProbeMatch>>();

    unique_probe_matches.into_iter().collect()
}
