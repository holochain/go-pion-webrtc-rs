//use crate::*;
use go_pion_webrtc_sys::API;

/// A go pion webrtc DataChannel.
#[derive(Debug)]
pub struct DataChannel(pub(crate) usize);

impl Drop for DataChannel {
    fn drop(&mut self) {
        unsafe { API.data_chan_free(self.0) }
    }
}
