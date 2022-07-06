#![deny(missing_docs)]
#![deny(warnings)]

//! Rust bindings to the go pion webrtc library.
//!
//! [![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
//! [![Forum](https://img.shields.io/badge/chat-forum%2eholochain%2enet-blue.svg?style=flat-square)](https://forum.holochain.org)
//! [![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
//! [![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

/// Error type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// Generic non-code-ed error type.
    Other(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<go_pion_webrtc_sys::Error> for Error {
    fn from(err: go_pion_webrtc_sys::Error) -> Self {
        Error::Other(err.error)
    }
}

impl From<Error> for go_pion_webrtc_sys::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Other(error) => Self { code: 0, error },
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

/// Go result type.
pub type Result<T> = std::result::Result<T, Error>;

mod evt;
pub use evt::*;

mod go_buf;
pub use go_buf::*;

mod peer_con;
pub use peer_con::*;

mod data_chan;
pub use data_chan::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    const STUN: &str = r#"{
    "iceServers": [
        {
            "urls": [
                "stun:stun.l.google.com:19302"
            ]
        }
    ]
}"#;

    #[test]
    fn peer_con() {
        let ice1 = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let ice2 = Arc::new(parking_lot::Mutex::new(Vec::new()));

        #[derive(Debug)]
        enum Cmd {
            Shutdown,
            ICE(String),
            Offer(String),
            Answer(String),
        }

        let (cmd_send_1, cmd_recv_1) = std::sync::mpsc::sync_channel(0);
        let cmd_send_1 = Arc::new(cmd_send_1);

        let (cmd_send_2, cmd_recv_2) = std::sync::mpsc::sync_channel(0);
        let cmd_send_2 = Arc::new(cmd_send_2);

        let hnd1 = {
            let cmd_send_2 = cmd_send_2.clone();
            let ice1 = ice1.clone();
            std::thread::spawn(move || {
                let mut peer1 = {
                    let cmd_send_2 = cmd_send_2.clone();
                    PeerConnection::new(STUN, move |evt| match evt {
                        PeerConnectionEvent::ICECandidate(candidate) => {
                            println!("peer1 in-ice: {}", candidate);
                            ice1.lock().push(candidate.clone());
                            cmd_send_2.send(Cmd::ICE(candidate)).unwrap();
                        }
                        PeerConnectionEvent::DataChannel(chan) => {
                            println!("peer1 in-chan: {:?}", chan);
                        }
                    })
                    .unwrap()
                };

                let _chan1 = peer1
                    .create_data_channel("{ \"label\": \"data\" }")
                    .unwrap();

                let offer = peer1.create_offer(None).unwrap();
                peer1.set_local_description(&offer).unwrap();
                cmd_send_2.send(Cmd::Offer(offer.clone())).unwrap();

                while let Ok(cmd) = cmd_recv_1.recv() {
                    match cmd {
                        Cmd::ICE(ice) => peer1.add_ice_candidate(&ice).unwrap(),
                        Cmd::Answer(answer) => {
                            peer1.set_remote_description(&answer).unwrap();
                        }
                        _ => break,
                    }
                }
            })
        };

        let hnd2 = {
            let cmd_send_1 = cmd_send_1.clone();
            let ice2 = ice2.clone();
            std::thread::spawn(move || {
                let mut peer2 = {
                    let cmd_send_1 = cmd_send_1.clone();
                    PeerConnection::new(STUN, move |evt| match evt {
                        PeerConnectionEvent::ICECandidate(candidate) => {
                            println!("peer2 in-ice: {}", candidate);
                            ice2.lock().push(candidate.clone());
                            cmd_send_1.send(Cmd::ICE(candidate)).unwrap();
                        }
                        PeerConnectionEvent::DataChannel(chan) => {
                            println!("peer2 in-chan: {:?}", chan);
                        }
                    })
                    .unwrap()
                };

                while let Ok(cmd) = cmd_recv_2.recv() {
                    match cmd {
                        Cmd::ICE(ice) => peer2.add_ice_candidate(&ice).unwrap(),
                        Cmd::Offer(offer) => {
                            peer2.set_remote_description(&offer).unwrap();
                            let answer = peer2.create_answer(None).unwrap();
                            peer2.set_local_description(&answer).unwrap();
                            cmd_send_1.send(Cmd::Answer(answer)).unwrap();
                        }
                        _ => break,
                    }
                }
            })
        };

        // -- wait -- //

        std::thread::sleep(std::time::Duration::from_secs(5));

        // -- cleanup -- //

        cmd_send_1.send(Cmd::Shutdown).unwrap();
        cmd_send_2.send(Cmd::Shutdown).unwrap();
        hnd1.join().unwrap();
        hnd2.join().unwrap();
    }
}
