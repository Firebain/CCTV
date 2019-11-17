use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use uuid::Uuid;
use xml::reader::{Error, EventReader, XmlEvent};

use crate::soap::prelude::*;
use crate::soap::{ProbeBuilder, SoapBuilderError};

const MULTICAST_ADDR: &str = "239.255.255.250:3702";

const DEVICE_TYPES: [&str; 3] = ["NetworkVideoTransmitter", "Device", "NetworkVideoDisplay"];
const READ_TIMEOUT: u64 = 300;
const RETRY_TIMES: usize = 3;

#[derive(Debug)]
pub struct ProbeMatch {
    name: String,
    xaddrs: Vec<String>,
}

struct ProbeMatchBuilder {
    urn: Option<String>,
    name: Option<String>,
    xaddrs: Option<Vec<String>>,
}

impl ProbeMatchBuilder {
    fn new() -> Self {
        Self {
            urn: None,
            name: None,
            xaddrs: None,
        }
    }

    fn urn(&mut self, urn: String) {
        self.urn = Some(urn);
    }

    fn name(&mut self, name: String) {
        self.name = Some(name);
    }

    fn xaddrs(&mut self, xaddrs: Vec<String>) {
        self.xaddrs = Some(xaddrs);
    }

    fn build(self) -> (String, ProbeMatch) {
        (
            self.urn.expect("Response doesn't contains urn"),
            ProbeMatch {
                name: self.name.expect("Response doesn't contains name"),
                xaddrs: self.xaddrs.expect("Response doesn't contains xaddrs"),
            },
        )
    }
}

pub fn discovery() -> Vec<ProbeMatch> {
    let socket = create_socket();

    multicast_probe_messages(&socket);

    let responses = recv_all_responses(&socket);

    parse_responses(responses).expect("Unexpected error while parsing responses")
}

fn create_socket() -> UdpSocket {
    let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
    let socket = UdpSocket::bind(free_socket_addr).expect("Could not bind to udp socket");

    let timeout = Duration::from_millis(READ_TIMEOUT);
    socket
        .set_read_timeout(Some(timeout))
        .expect("set_read_timeout call failed");

    socket
}

fn multicast_probe_messages(socket: &UdpSocket) {
    let multicast_addr: SocketAddr = MULTICAST_ADDR
        .parse()
        .expect("Error while parsing multicast addr");

    let messages = DEVICE_TYPES
        .iter()
        .map(|device_type| ProbeBuilder::new(device_type, Uuid::new_v4()).build())
        .collect::<Result<Vec<String>, SoapBuilderError>>()
        .expect("Error while building Probe XML");

    for message in messages {
        for _ in 0..RETRY_TIMES {
            socket
                .send_to(message.as_bytes(), multicast_addr)
                .expect("couldn't send data");
        }
    }
}

fn recv_all_responses(socket: &UdpSocket) -> Vec<String> {
    let mut responses = Vec::new();
    loop {
        let mut buf = [0; 65_535];

        match socket.recv(&mut buf) {
            Ok(amt) => {
                let string = String::from_utf8(buf[..amt].to_vec())
                    .expect("Response contains non utf-8 characters");

                responses.push(string);
            }
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => break,
                _ => panic!("Unexpected error while receiving new messages"),
            },
        }
    }

    responses
}

fn parse_responses(responses: Vec<String>) -> Result<Vec<ProbeMatch>, Error> {
    let response = responses.first().unwrap();

    let mut parser = EventReader::from_str(&response);

    let mut probe_matches = HashMap::new();
    loop {
        let event = parser.next()?;

        match event {
            XmlEvent::StartElement { name, .. } => {
                if let "ProbeMatch" = name.local_name.as_str() {
                    let (urn, probe_match) = parse_probe_match(&mut parser)?;

                    probe_matches.insert(urn, probe_match);
                }
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    let probe_matches = probe_matches.into_iter().map(|(_, val)| val).collect();

    Ok(probe_matches)
}

fn parse_probe_match(parser: &mut EventReader<&[u8]>) -> Result<(String, ProbeMatch), Error> {
    let mut probe_match_builder = ProbeMatchBuilder::new();

    loop {
        let event = parser.next()?;

        match event {
            XmlEvent::StartElement { name, .. } => match name.local_name.as_str() {
                "EndpointReference" => {
                    let event = parser.next()?;

                    if let XmlEvent::StartElement { name, .. } = event {
                        if let "Address" = name.local_name.as_str() {
                            let event = parser.next()?;

                            if let XmlEvent::Characters(urn) = event {
                                probe_match_builder.urn(urn);
                            }
                        }
                    }
                }
                "Scopes" => {
                    let event = parser.next()?;

                    if let XmlEvent::Characters(scopes) = event {
                        let scopes = scopes.split(' ');

                        for scope in scopes {
                            if scope.starts_with("onvif://www.onvif.org/name") {
                                let parts: Vec<&str> = scope.split('/').collect();

                                let name = parts[parts.len() - 1];

                                probe_match_builder.name(name.to_string());
                            }
                        }
                    }
                }
                "XAddrs" => {
                    let event = parser.next()?;
                    if let XmlEvent::Characters(xaddrs) = event {
                        let xaddrs = xaddrs
                            .split(' ')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();

                        probe_match_builder.xaddrs(xaddrs);
                    }
                }
                _ => {}
            },
            XmlEvent::EndElement { name } => {
                if let "ProbeMatch" = name.local_name.as_str() {
                    break;
                }
            }
            _ => {}
        }
    }

    Ok(probe_match_builder.build())
}
