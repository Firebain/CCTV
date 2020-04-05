use std::io::prelude::*;

use std::collections::HashMap;
use std::net::TcpStream;
use url::Url;

const REQUIRED_METHODS: [&str; 5] = ["OPTIONS", "DESCRIBE", "SETUP", "PLAY", "TEARDOWN"];

pub struct Client {
    connection: TcpStream,
    url: String,
    cseq: u32,
}

impl Client {
    pub fn connect(url: String) -> Self {
        let parsed_url = Url::parse(&url).unwrap();

        let addrs = parsed_url.socket_addrs(|| None).unwrap();

        let mut client = Self {
            connection: TcpStream::connect(&*addrs).unwrap(),
            url,
            cseq: 1,
        };

        let methods = client.options();
        let contains_required_methods = REQUIRED_METHODS
            .iter()
            .all(|item| methods.contains(&(*item).to_string()));

        if contains_required_methods {
            client
        } else {
            panic!("RTSP server doesn't contains required methods")
        }
    }

    pub fn describe(&mut self) {
        let mut headers = self.default_headers("DESCRIBE");
        headers.push("Accept: application/sdp".to_string());

        self.write(headers);

        self.recv();
    }

    pub fn setup(&mut self, main_socket_port: u16, second_socket_port: u16) -> String {
        let mut headers = self.default_headers("SETUP");
        headers.push(format!(
            "Transport: RTP/AVP;unicast;client_port={}-{}",
            main_socket_port, second_socket_port
        ));

        self.write(headers);

        let (mut headers, _) = self.recv();
        headers.remove("Session").unwrap()
    }

    pub fn play(&mut self, session: &str) {
        let mut headers = self.default_headers("PLAY");
        headers.push(format!("Session: {}", session));
        headers.push("Range: npt=0.000-".to_string());

        self.write(headers);

        self.recv();
    }

    // pub fn teardown(&mut self, session: &str) {
    //     let mut headers = self.default_headers("TEARDOWN");
    //     headers.push(format!("Session: {}", session));

    //     self.write(headers);

    //     self.recv();
    // }

    pub fn options(&mut self) -> Vec<String> {
        let options = self.default_headers("OPTIONS");

        self.write(options);

        let (headers, _) = self.recv();
        let public_methods = headers.get("Public").unwrap();

        public_methods
            .split(',')
            .map(|item| item.trim().to_string())
            .collect()
    }

    fn write(&mut self, headers: Vec<String>) {
        let headers = headers.join("\r\n") + "\r\n\r\n";

        let headers = headers.as_bytes();

        self.connection.write_all(headers).unwrap();
    }

    fn recv(&mut self) -> (HashMap<String, String>, String) {
        let mut buf = [0; 65_535];

        let amt = self.connection.read(&mut buf).unwrap();
        let res = String::from_utf8(Vec::from(&buf[..amt])).unwrap();

        let mut data = res.split("\r\n\r\n");
        let headers = data.next().unwrap();
        let body = data.next().unwrap().to_string();

        let mut headers = headers.split("\r\n");
        let status = headers.next().unwrap();

        let status_code = status.split(' ').nth(1).unwrap();

        if status_code != "200" {
            panic!("Wrong status code");
        }

        let headers = headers.map(|el| {
            let mut split = el.split(": ");

            Some((split.next()?.to_string(), split.next()?.to_string()))
        });

        let headers: Option<HashMap<String, String>> = headers.collect();
        let headers = headers.unwrap();

        let cseq = headers.get("CSeq").unwrap().parse::<u32>().unwrap();

        if cseq == self.cseq {
            self.cseq += 1;

            (headers, body)
        } else {
            panic!("Responce CSeq is different");
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
