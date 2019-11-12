mod soap;

use std::io::ErrorKind;
use std::time::Duration;
use soap::{ProbeBuilder, SoapBuilderError};
use std::net::{SocketAddr, UdpSocket};
use uuid::Uuid;
use soap::prelude::*;
use xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;

#[derive(Debug)]
struct ProbeMatch {
    name: String,
    xaddr: String
}

impl ProbeMatch {
    fn new(name: String, xaddr: String) -> Self {
        Self {
            name,
            xaddr
        }
    }
}

#[derive(Debug)]
struct ProbeMatchBuilder {
    name: Option<String>,
    urn: Option<String>,
    xaddr: Option<String>
}

impl ProbeMatchBuilder {
    fn new() -> Self {
        Self {
            name: None,
            urn: None,
            xaddr: None
        }
    }

    fn build(self) -> (String, ProbeMatch) {
        (self.urn.unwrap(), ProbeMatch::new(self.name.unwrap(), self.xaddr.unwrap()))
    }
}

fn main() {
    let device_types = vec!["NetworkVideoTransmitter", "Device", "NetworkVideoDisplay"];

    const DISCOVERY_RETRY: usize = 1;

    let messages = device_types
        .iter()
        .map(|device_type| {
            ProbeBuilder::new(device_type, Uuid::new_v4()).build()
        })
        .collect::<Result<Vec<String>, SoapBuilderError>>()
        .unwrap();

    let multicast_addr: SocketAddr = "239.255.255.250:3702".parse().unwrap();

    let all_interfaces = SocketAddr::from(([0, 0, 0, 0], 0));

    let socket = UdpSocket::bind(&all_interfaces).expect("Could not bind to udp socket");

    socket.set_read_timeout(Some(Duration::from_millis(300))).expect("set_read_timeout call failed");

    for message in messages {
        for _ in 0 .. DISCOVERY_RETRY {
            socket.send_to(message.as_bytes(), multicast_addr).unwrap();
        }
    }

    let mut responses = Vec::new();
    loop {
        let mut buf = [0; 65_535];

        match socket.recv(&mut buf) {
            Ok(amt) => responses.push(String::from_utf8(buf[..amt].to_vec()).unwrap()),
            Err(err) => {
                match err.kind() {
                    ErrorKind::WouldBlock => break,
                    _ => panic!("Unexpected error")
                }
            }
        }
    }

    let mut probe_matches: HashMap<String, ProbeMatch> = HashMap::new();

    for response in responses {
        let mut parser = EventReader::from_str(&response);

        let mut probe_match_builder = ProbeMatchBuilder::new();

        loop {
            let event = parser.next().unwrap();

            match event {
                XmlEvent::StartElement { name, .. } => {
                    match name.local_name.as_str() {
                        "EndpointReference" => {
                            let event = parser.next().unwrap();

                            if let XmlEvent::StartElement { name, .. } = event {
                                match name.local_name.as_str() {
                                    "Address" => {
                                        let event = parser.next().unwrap();

                                        if let XmlEvent::Characters(urn) = event {
                                            probe_match_builder.urn = Some(urn);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        },
                        "XAddrs" => {
                            let event = parser.next().unwrap();
                            
                            if let XmlEvent::Characters(xaadrs) = event {
                                probe_match_builder.xaddr = Some(String::from(xaadrs.trim()));
                            }
                        },
                        "Scopes" => {
                            let event = parser.next().unwrap();

                            if let XmlEvent::Characters(scopes) = event {
                                let scopes = scopes.split(' ');

                                for scope in scopes {
                                    if scope.starts_with("onvif://www.onvif.org/name") {
                                        let parts: Vec<&str> = scope.split('/').collect();

                                        let name = parts[parts.len() - 1];

                                        probe_match_builder.name = Some(String::from(name));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                },
                XmlEvent::EndDocument => {
                    break;
                },
                _ => {}
            }
        }

        let (urn, probe_match) = probe_match_builder.build();

        probe_matches.insert(urn, probe_match);
    }

    let probe_matches: Vec<ProbeMatch> = probe_matches.into_iter().map(|(_, val)| val).collect();

    dbg!(probe_matches);
}
