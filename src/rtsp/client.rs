use std::error;
use std::fmt;
use std::io::prelude::*;
use std::io::Error as IoError;
use std::string::FromUtf8Error;

use std::net::{TcpStream, UdpSocket};
use url::{ParseError, Url};

const REQUIRED_METHODS: [&str; 4] = ["DESCRIBE", "SETUP", "PLAY", "TEARDOWN"];

pub struct RTSPClient {
    connection: TcpStream,
    url: String,
    cseq: u32,
}

impl RTSPClient {
    pub fn connect(url: String) -> Result<Self, RTSPClientError> {
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

        let methods = client.options()?;
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
        let describe = format!(
            "{}\
            Accept: application/sdp\r\n", self.default_headers("DESCRIBE"));

        self.write(describe)?;

        self.recv()?;

        Ok(())
    }

    pub fn setup(&mut self, main_socket: UdpSocket, second_socket: UdpSocket) -> Result<String, RTSPClientError> {
        let setup = format!(
            "{}\
            Transport: RTP/AVP;unicast;client_port={}-{}\r\n\
            \r\n", 
            self.default_headers("SETUP"), 
            main_socket.local_addr()?.port(),
            second_socket.local_addr()?.port());

        self.write(setup)?;

        let res = self.recv()?;
        let session = res.iter().find(|item| item.starts_with("Session"));

        let session = match session {
            Some(session) => session,
            None => return Err(RTSPClientError::UnexpectedError("SETUP response doesn't contains Session header"))
        };

        Ok(session
            .split_at(8)
            .1
            .trim()
            .to_string())
    }

    pub fn play(&mut self, session: &String) -> Result<(), RTSPClientError> {
        let play = format!(
            "{}\
            Session: {}\r\n\
            Range: npt=0.000-\r\n\
            \r\n", 
            self.default_headers("PLAY"), 
            session);

        self.write(play)?;

        self.recv()?;

        Ok(())
    }

    pub fn teardown(&mut self, session: &String) -> Result<(), RTSPClientError> {
        let teardown = format!(
            "{}\
            Session: {}\r\n\
            \r\n", 
            self.default_headers("TEARDOWN"), 
            session);

        self.write(teardown)?;

        self.recv()?;

        Ok(())
    }

    pub fn options(&mut self) -> Result<Vec<String>, RTSPClientError> {
        let options = format!("{}\r\n", self.default_headers("OPTIONS"));

        self.write(options)?;

        let res = self.recv()?;
        let public_methods = res.iter().find(|item| item.starts_with("Public"));

        match public_methods {
            Some(methods) => Ok(methods
                .split_at(8)
                .1
                .split(",")
                .map(|item| item.trim())
                .map(|item| item.to_string())
                .collect()),
            None => Err(RTSPClientError::UnexpectedError(
                "OPTIONS response doesn't contains Public methods",
            )),
        }
    }

    fn write(&mut self, data: String) -> Result<(), RTSPClientError> {
        println!("C->S:\r\n{}", data);

        let data = data.as_bytes();
        self.connection.write(data)?;

        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<String>, RTSPClientError> {
        let mut buf = [0; 65_535];

        let amt = self.connection.read(&mut buf)?;
        let res = String::from_utf8(Vec::from(&buf[..amt]))?;

        println!("S->C:\r\n{}", res);

        let mut res = res
            .split("\n")
            .map(|item| item.trim_end())
            .map(|item| item.to_string());

        let cseq = res
            .find(|item| item.starts_with("CSeq"))
            .ok_or(RTSPClientError::UnexpectedError("RTSP responce doesn't contains CSeq"))?;

        let cseq = cseq.split_at(5).1.trim().parse::<u32>()
            .map_err(|_| RTSPClientError::UnexpectedError("Error while parsing RTSP responce CSeq"))?;

        if cseq == self.cseq {
            self.cseq = self.cseq + 1;

            Ok(res.collect())
        } else {
            Err(RTSPClientError::UnexpectedError(
                "Responce CSeq is different",
            ))
        }
    }

    fn default_headers(&self, method: &str) -> String {
        format!(
            "{} {} RTSP/1.0\r\n\
             CSeq: {}\r\n\
             User-Agent: Rust RTSP client\r\n",
            method, self.url, self.cseq
        )
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
