use crate::protos::hoymiles::RealData::HMSStateResponse;
use crate::RealData::RealDataResDTO;
use crc16::*;
use log::{debug,info};
use protobuf::Message;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

static INVERTER_PORT: u16 = 10081;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkState {
    Unknown,
    Online,
    Offline,
}

pub struct Inverter<'a> {
    host: &'a str,
    pub state: NetworkState,
    sequence: u16,
}

impl<'a> Inverter<'a> {
    pub fn new(host: &'a str) -> Self {
        Self {
            host,
            state: NetworkState::Unknown,
            sequence: 0_u16,
        }
    }

    fn set_state(&mut self, new_state: NetworkState) {
        if self.state != new_state {
            self.state = new_state;
            info!("Inverter is {new_state:?}");
        }
    }

    pub fn update_state(&mut self) -> Option<HMSStateResponse> {
        self.sequence = self.sequence.wrapping_add(1);

        let /*mut*/ request = RealDataResDTO::default();
        // let date = Local::now();
        // let time_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        // request.ymd_hms = time_string;
        // request.cp = 23 + sequence as i32;
        // request.offset = 0;
        // request.time = epoch();
        let header = b"\x48\x4d\xa3\x03";
        let request_as_bytes = request.write_to_bytes().expect("serialize to bytes");
        let crc16 = State::<MODBUS>::calculate(&request_as_bytes);
        let len = request_as_bytes.len() as u16 + 10u16;

        // compose request message
        let mut message = Vec::new();
        message.extend_from_slice(header);
        message.extend_from_slice(&self.sequence.to_be_bytes());
        message.extend_from_slice(&crc16.to_be_bytes());
        message.extend_from_slice(&len.to_be_bytes());
        message.extend_from_slice(&request_as_bytes);

        let ip = self.host.parse().expect("Unable to parse socket address");
        let address = SocketAddr::new(IpAddr::V4(ip), INVERTER_PORT);
        let stream = TcpStream::connect_timeout(&address, Duration::from_millis(500));
        if let Err(e) = stream {
            debug!("s {e}");
            self.set_state(NetworkState::Offline);
            return None;
        }
        let mut stream = stream.unwrap();
        stream.set_write_timeout(Some(Duration::new(45, 0))).expect("set_write_timeout call failed");
        if let Err(e) = stream.write(&message) {
            info!(r#"{e}"#);
            self.set_state(NetworkState::Offline);
            return None;
        }
        let mut buf = [0u8; 1024];
        stream.set_read_timeout(Some(Duration::new(45, 0))).expect("set_read_timeout call failed");
        let read = stream.read(&mut buf);
        if let Err(e) = read {
            info!("r {e}");
            self.set_state(NetworkState::Offline);
            return None;
        }
        let parsed = HMSStateResponse::parse_from_bytes(&buf[10..read.unwrap()]);
        if let Err(e) = parsed {
            info!("p {e}");
            self.set_state(NetworkState::Offline);
            return None;
        }
        debug_assert!(parsed.is_ok());
        let response = parsed.unwrap();
        self.set_state(NetworkState::Online);
        Some(response)
    }
}
