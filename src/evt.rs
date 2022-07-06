use crate::*;
use go_pion_webrtc_sys::Event as SysEvent;
use go_pion_webrtc_sys::API;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Incoming events related to a PeerConnection.
#[derive(Debug)]
pub enum PeerConnectionEvent {
    /// Received a trickle ICE candidate.
    ICECandidate(String),

    /// Received an incoming data channel.
    DataChannel(DataChannel),
}

pub(crate) fn register_peer_con_evt_cb(id: usize, cb: PeerConEvtCb) {
    MANAGER.lock().peer_con.insert(id, cb);
}

pub(crate) fn unregister_peer_con_evt_cb(id: usize) {
    MANAGER.lock().peer_con.remove(&id);
}

pub(crate) type PeerConEvtCb =
    Arc<dyn Fn(PeerConnectionEvent) + 'static + Send + Sync>;

static MANAGER: Lazy<Mutex<Manager>> = Lazy::new(|| {
    unsafe {
        API.on_event(|sys_evt| match sys_evt {
            SysEvent::Error(_error) => (),
            SysEvent::PeerConICECandidate {
                peer_con_id,
                candidate,
            } => {
                let man = MANAGER.lock();
                if let Some(cb) = man.peer_con.get(&peer_con_id) {
                    cb(PeerConnectionEvent::ICECandidate(candidate));
                }
            }
            SysEvent::PeerConStateChange {
                peer_con_id: _,
                peer_con_state: _,
            } => (),
            SysEvent::PeerConDataChan {
                peer_con_id,
                data_chan_id,
            } => {
                let man = MANAGER.lock();
                if let Some(cb) = man.peer_con.get(&peer_con_id) {
                    cb(PeerConnectionEvent::DataChannel(DataChannel(
                        data_chan_id,
                    )))
                }
            }
        });
    }
    Manager::new()
});

struct Manager {
    peer_con: HashMap<usize, PeerConEvtCb>,
}

impl Manager {
    pub fn new() -> Mutex<Self> {
        Mutex::new(Self {
            peer_con: HashMap::new(),
        })
    }
}
