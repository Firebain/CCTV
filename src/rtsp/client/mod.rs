use std::error;
use std::fmt;
use std::io::prelude::*;
use std::io::Error as IoError;
use std::string::FromUtf8Error;

use std::collections::HashMap;
use std::net::TcpStream;
use url::{ParseError, Url};

const REQUIRED_METHODS: [&str; 5] = ["OPTIONS", "DESCRIBE", "SETUP", "PLAY", "TEARDOWN"];

pub struct RTSPClient {
    connection: TcpStream,
    url: String,
    cseq: u32,
}

impl RTSPClient {
    pub async fn connect(url: String) -> Result<Self, RTSPClientError> {
        let parsed_url = Url::parse(&url)?;
        if parsed_url.scheme() != "rtsp" {
            return Err(RTSPClientError::WrongUrl("url sheme is not rtsp"));
        }

        let addrs = parsed_url.socket_addrs(|| None)?;

        let mut client = RTSPClient {
            connection: TcpStream::connect(&*addrs)?,
            url,
            cseq: 1,
        };

        let methods = client.options().await?;
        let contains_required_methods = REQUIRED_METHODS
            .iter()
            .all(|item| methods.contains(&item.to_string()));

        if contains_required_methods {
            Ok(client)
        } else {
            Err(RTSPClientError::UnexpectedError(
                "RTSP server doesn't contains required methods",
            ))
        }
    }

    pub fn describe(&mut self) -> Result<(), RTSPClientError> {
        let mut headers = self.default_headers("DESCRIBE");
        headers.push("Accept: application/sdp".to_string());

        self.write(headers)?;

        self.recv()?;

        Ok(())
    }

    pub fn setup(
        &mut self,
        main_socket_port: u16,
        second_socket_port: u16,
    ) -> Result<String, RTSPClientError> {
        let mut headers = self.default_headers("SETUP");
        headers.push(format!(
            "Transport: RTP/AVP;unicast;client_port={}-{}",
            main_socket_port, second_socket_port
        ));

        self.write(headers)?;

        let (mut headers, _) = self.recv()?;
        let session = headers.remove("Session");

        match session {
            Some(session) => Ok(session),
            None => Err(RTSPClientError::UnexpectedError(
                "SETUP response doesn't contains Session header",
            )),
        }
    }

    pub fn play(&mut self, session: &str) -> Result<(), RTSPClientError> {
        let mut headers = self.default_headers("PLAY");
        headers.push(format!("Session: {}", session));
        headers.push("Range: npt=0.000-".to_string());

        self.write(headers)?;

        self.recv()?;

        Ok(())
    }

    pub fn teardown(&mut self, session: &str) -> Result<(), RTSPClientError> {
        let mut headers = self.default_headers("TEARDOWN");
        headers.push(format!("Session: {}", session));

        self.write(headers)?;

        self.recv()?;

        Ok(())
    }

    pub async fn options(&mut self) -> Result<Vec<String>, RTSPClientError> {
        let options = self.default_headers("OPTIONS");

        self.write(options)?;

        let (headers, _) = self.recv()?;
        let public_methods = headers.get("Public");

        match public_methods {
            Some(methods) => {
                let methods = methods
                    .split(',')
                    .map(|item| item.trim().to_string())
                    .collect();

                Ok(methods)
            }
            None => Err(RTSPClientError::UnexpectedError(
                "OPTIONS response doesn't contains Public methods",
            )),
        }
    }

    fn write(&mut self, headers: Vec<String>) -> Result<(), RTSPClientError> {
        let headers = headers.join("\r\n") + "\r\n\r\n";

        let headers = headers.as_bytes();

        self.connection.write_all(headers)?;

        Ok(())
    }

    fn recv(&mut self) -> Result<(HashMap<String, String>, String), RTSPClientError> {
        let mut buf = [0; 65_535];

        let amt = self.connection.read(&mut buf)?;
        let res = String::from_utf8(Vec::from(&buf[..amt]))?;

        let mut data = res.split("\r\n\r\n");
        let headers = data
            .nth(0)
            .ok_or(RTSPClientError::UnexpectedError("Headers is missing"))?;
        let body = data
            .nth(0)
            .ok_or(RTSPClientError::UnexpectedError("Body is missing"))?
            .to_string();

        let mut headers = headers.split("\r\n");
        let status = headers
            .nth(0)
            .ok_or(RTSPClientError::UnexpectedError("Headers is missing"))?;

        let status_code = status
            .split(' ')
            .nth(1)
            .ok_or(RTSPClientError::UnexpectedError("Status code is missing"))?;

        if status_code != "200" {
            return Err(RTSPClientError::UnexpectedError("Wrong status code"));
        }

        let headers = headers.map(|el| {
            let mut split = el.split(": ");

            Some((split.nth(0)?.to_string(), split.nth(0)?.to_string()))
        });

        let headers: Option<HashMap<String, String>> = headers.collect();
        let headers = headers.ok_or(RTSPClientError::UnexpectedError(
            "Error while parsing headers",
        ))?;

        let cseq = headers
            .get("CSeq")
            .ok_or(RTSPClientError::UnexpectedError(
                "RTSP responce doesn't contains CSeq",
            ))?
            .parse::<u32>()
            .map_err(|_| {
                RTSPClientError::UnexpectedError("Error while parsing RTSP responce CSeq")
            })?;

        if cseq == self.cseq {
            self.cseq += 1;

            Ok((headers, body))
        } else {
            Err(RTSPClientError::UnexpectedError(
                "Responce CSeq is different",
            ))
        }
    }

    fn default_headers(&self, method: &str) -> Vec<String> {
        vec![
            format!("{} {} RTSP/1.0", method, self.url),
            format!("CSeq: {}", self.cseq),
            "User-Agent: Rust RTSP client".to_string(),
        ]
    }
}

#[derive(Debug)]
pub enum RTSPClientError {
    ParsingUrl(ParseError),
    WrongUrl(&'static str),
    ConnectionError(IoError),
    UnexpectedError(&'static str),
}

impl From<ParseError> for RTSPClientError {
    fn from(err: ParseError) -> Self {
        Self::ParsingUrl(err)
    }
}

impl From<IoError> for RTSPClientError {
    fn from(err: IoError) -> Self {
        Self::ConnectionError(err)
    }
}

impl From<FromUtf8Error> for RTSPClientError {
    fn from(_: FromUtf8Error) -> Self {
        Self::UnexpectedError("None utf 8 string is received")
    }
}

impl fmt::Display for RTSPClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParsingUrl(err) => write!(f, "Error while parsing url: {}", err),
            Self::ConnectionError(err) => write!(f, "Connection error: {}", err),
            Self::WrongUrl(err) => write!(f, "Url is wrong: {}", err),
            Self::UnexpectedError(err) => write!(f, "UnexpectedError: {}", err),
        }
    }
}

impl error::Error for RTSPClientError {}
