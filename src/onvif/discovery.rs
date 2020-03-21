use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::{Error as IOError, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::{error, fmt, result};

use serde::{Deserialize, Serialize};
use serde_xml_rs::Error as XmlParsingError;
use uuid::Uuid;

use super::soap::headers::Probe;
use super::soap::Client;
use super::soap::Envelope;
use crate::xml::Result as XmlBuilderResult;

const MULTICAST_ADDR: &str = "239.255.255.250:3702";

const DEVICE_TYPES: [&str; 3] = ["NetworkVideoTransmitter", "Device", "NetworkVideoDisplay"];
const READ_TIMEOUT: u64 = 300;
const RETRY_TIMES: usize = 3;

type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParsingError(XmlParsingError),
    IOError(IOError),
    UnexpectedError(&'static str),
}

impl From<XmlParsingError> for Error {
    fn from(err: XmlParsingError) -> Self {
        Self::ParsingError(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParsingError(err) => write!(f, "Parsing err: {}", err),
            Self::IOError(err) => write!(f, "IO err: {}", err),
            Self::UnexpectedError(err) => write!(f, "Unexpected err: {}", err),
        }
    }
}

impl error::Error for Error {}

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

impl TryFrom<RawProbeMatch> for ProbeMatch {
    type Error = Error;

    fn try_from(raw_probe_match: RawProbeMatch) -> Result<Self> {
        let id = raw_probe_match.endpoint_reference.address[9..].to_string();

        let name = raw_probe_match
            .scopes
            .split(' ')
            .find(|scope| scope.starts_with("onvif://www.onvif.org/name"))
            .ok_or(Error::UnexpectedError("Name scope is missing"))?
            .split('/')
            .last()
            .ok_or(Error::UnexpectedError("Name scope is empty"))?
            .to_string();

        let xaddrs: Vec<String> = raw_probe_match
            .xaddrs
            .split(' ')
            .map(|xaddr| xaddr.to_string())
            .collect();

        Ok(Self { name, id, xaddrs })
    }
}

pub async fn discovery() -> Result<Vec<ProbeMatch>> {
    let socket = create_socket()?;

    multicast_probe_messages(&socket)?;

    let responses = recv_all_responses(&socket)?;

    Ok(parse_responses(responses)?)
}

fn create_socket() -> Result<UdpSocket> {
    let free_socket_addr = SocketAddr::from(([0, 0, 0, 0], 0));
    let socket = UdpSocket::bind(free_socket_addr)?;

    let timeout = Duration::from_millis(READ_TIMEOUT);
    socket.set_read_timeout(Some(timeout))?;

    Ok(socket)
}

fn multicast_probe_messages(socket: &UdpSocket) -> Result<()> {
    let multicast_addr: SocketAddr = MULTICAST_ADDR
        .parse()
        .map_err(|_| Error::UnexpectedError("Error while parsing multicast addr"))?;

    let messages = DEVICE_TYPES
        .iter()
        .map(|device_type| {
            let client = Client {
                header: Probe::new(Uuid::new_v4()),
            };

            client.build(|writer| {
                writer
                    .new_event("d:Probe")
                    .ns("d", "http://schemas.xmlsoap.org/ws/2005/04/discovery")
                    .write()?;

                writer
                    .new_event("d:Types")
                    .ns("dp0", "http://www.onvif.org/ver10/network/wsdl")
                    .content(&format!("dp0:{}", device_type))
                    .end()?;

                writer.end_event()?; // Probe

                Ok(())
            })
        })
        .collect::<XmlBuilderResult<Vec<String>>>()
        .map_err(|_| Error::UnexpectedError("Xml builder error"))?;

    for message in messages {
        for _ in 0..RETRY_TIMES {
            socket.send_to(message.as_bytes(), multicast_addr)?;
        }
    }

    Ok(())
}

fn recv_all_responses(socket: &UdpSocket) -> Result<Vec<String>> {
    let mut responses = Vec::new();
    loop {
        let mut buf = [0; 65_535];

        // TODO: Probably it can be async
        match socket.recv(&mut buf) {
            Ok(amt) => {
                let string = String::from_utf8(buf[..amt].to_vec())
                    .map_err(|_| Error::UnexpectedError("Response contains none utf-8 chars"))?;

                responses.push(string);
            }
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => break,
                _ => return Err(Error::IOError(err)),
            },
        }
    }

    Ok(responses)
}

fn parse_responses(responses: Vec<String>) -> Result<Vec<ProbeMatch>> {
    let parsed_responses = responses
        .into_iter()
        .map(|response| serde_xml_rs::from_str(&response))
        .map(|result| result.map_err(|err| Error::ParsingError(err)))
        .collect::<Result<Vec<Envelope<DiscoveryBody>>>>()?;

    let unique_probe_matches = parsed_responses
        .into_iter()
        .flat_map(|response| response.body.probe_matches_container.probe_matches)
        .map(ProbeMatch::try_from)
        .collect::<Result<HashSet<ProbeMatch>>>()?;

    let probe_matches = unique_probe_matches
        .into_iter()
        .collect::<Vec<ProbeMatch>>();

    Ok(probe_matches)
}
