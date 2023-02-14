use crate::data;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

pub struct Channels {
    pub connect_rx: mpsc::Receiver<String>,

    pub plot_tx: mpsc::Sender<data::Point>,
    pub connection_state_tx: mpsc::Sender<ConnectionState>,
}

struct Adsb {
    channels: Channels,
}

impl Adsb {
    fn new(channels: Channels) -> Self {
        Adsb { channels }
    }

    fn run(&mut self) {
        loop {
            self.set_connection_state(ConnectionState::Disconnected);
            let addr = self.channels.connect_rx.recv().unwrap();
            self.set_connection_state(ConnectionState::Connecting);

            if let Ok(stream) = TcpStream::connect(addr) {
                stream
                    .set_read_timeout(Option::from(Duration::from_secs(30)))
                    .unwrap();
                self.set_connection_state(ConnectionState::Connected);
                self.read(stream);
            } else {
                println!("Couldn't connect to server...");
                continue;
            }
        }
    }

    fn read(&mut self, stream: TcpStream) {
        let mut reader = BufReader::new(stream);

        loop {
            let mut message = String::new();

            let result = reader.read_line(&mut message);
            match result {
                Ok(_) => {
                    self.on_message(&message);
                }
                Err(e) => {
                    println!("error reading: {}", e);
                    return;
                }
            }
        }

        // self.add_point(rng.gen_range(0..60000));
        // thread::sleep(time::Duration::from_millis(25));
    }

    //*5124;
    fn on_message(&mut self, message: &str) {
        if message.len() != 7 {
            // Only want mode A/C messages
            return;
        }

        if let Ok(i) = u32::from_str_radix(&message[1..5], 16) {
            if let Ok(alt) = mode_a_to_mode_c(i) {
                self.add_point(alt * 100);
            }
        }
    }

    fn add_point(&mut self, height: u32) {
        let now = SystemTime::now();

        self.channels
            .plot_tx
            .send(data::Point { height, time: now })
            .expect("Failed to send plot");
    }

    fn set_connection_state(&mut self, state: ConnectionState) {
        self.channels
            .connection_state_tx
            .send(state)
            .expect("Failed to update connection state");
    }
}

pub fn run(channels: Channels) {
    let mut adsb = Adsb::new(channels);
    adsb.run();
}

// Taken from: https://github.com/rsadsb/adsb_deku/blob/c9944134ef5816f1f2151d8a7ac7f5556f213e91/libadsb_deku/src/mode_ac.rs#L53 under the MIT license
fn mode_a_to_mode_c(mode_a: u32) -> Result<u32, &'static str> {
    let mut five_hundreds: u32 = 0;
    let mut one_hundreds: u32 = 0;

    // check zero bits are zero, D1 set is illegal; C1,,C4 cannot be Zero
    if (mode_a & 0xffff_8889) != 0 || (mode_a & 0x0000_00f0) == 0 {
        return Err("Invalid altitude");
    }

    if mode_a & 0x0010 != 0 {
        one_hundreds ^= 0x007;
    } // C1
    if mode_a & 0x0020 != 0 {
        one_hundreds ^= 0x003;
    } // C2
    if mode_a & 0x0040 != 0 {
        one_hundreds ^= 0x001;
    } // C4

    // Remove 7s from OneHundreds (Make 7->5, snd 5->7).
    if (one_hundreds & 5) == 5 {
        one_hundreds ^= 2;
    }

    // Check for invalid codes, only 1 to 5 are valid
    if one_hundreds > 5 {
        return Err("Invalid altitude");
    }

    // if mode_a & 0x0001 {five_hundreds ^= 0x1FF;} // D1 never used for altitude
    if mode_a & 0x0002 != 0 {
        five_hundreds ^= 0x0ff;
    } // D2
    if mode_a & 0x0004 != 0 {
        five_hundreds ^= 0x07f;
    } // D4

    if mode_a & 0x1000 != 0 {
        five_hundreds ^= 0x03f;
    } // A1
    if mode_a & 0x2000 != 0 {
        five_hundreds ^= 0x01f;
    } // A2
    if mode_a & 0x4000 != 0 {
        five_hundreds ^= 0x00f;
    } // A4

    if mode_a & 0x0100 != 0 {
        five_hundreds ^= 0x007;
    } // B1
    if mode_a & 0x0200 != 0 {
        five_hundreds ^= 0x003;
    } // B2
    if mode_a & 0x0400 != 0 {
        five_hundreds ^= 0x001;
    } // B4

    // Correct order of one_hundreds.
    if five_hundreds & 1 != 0 && one_hundreds <= 6 {
        one_hundreds = 6 - one_hundreds;
    }

    let n = (five_hundreds * 5) + one_hundreds;
    if n >= 13 {
        Ok(n - 13)
    } else {
        Err("Invalid altitude")
    }
}
