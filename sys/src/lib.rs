//! Rust bindings to the go pion webrtc library.
//!
//! [![Project](https://img.shields.io/badge/project-holochain-blue.svg?style=flat-square)](http://holochain.org/)
//! [![Forum](https://img.shields.io/badge/chat-forum%2eholochain%2enet-blue.svg?style=flat-square)](https://forum.holochain.org)
//! [![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.org)
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
//! [![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
//!
//! Access the go-pion-webrtc api interface using the
//! pub once_cell::sync::Lazy static [API] handle.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use once_cell::sync::Lazy;
use std::sync::Arc;

#[cfg(target_os = "macos")]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.dylib"));

#[cfg(target_os = "windows")]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.dll"));

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.so"));

/// Constants
pub mod constants;
use constants::*;

#[ouroboros::self_referencing]
struct LibInner {
    _temp_path: tempfile::TempPath,
    lib: libloading::Library,
    #[borrows(lib)]
    // not 100% sure about this, but we never unload the lib,
    // so it's effectively 'static
    #[covariant]
    on_event: libloading::Symbol<
        'this,
        unsafe extern "C" fn(
            // event_cb
            Option<
                unsafe extern "C" fn(
                    *mut libc::c_void, // response_usr
                    usize,             // response_type
                    usize,             // slot_a
                    usize,             // slot_b
                    usize,             // slot_c
                    usize,             // slot_d
                ),
            >,
            *mut libc::c_void, // event_usr
        ) -> *mut libc::c_void,
    >,
    #[borrows(lib)]
    // not 100% sure about this, but we never unload the lib,
    // so it's effectively 'static
    #[covariant]
    call: libloading::Symbol<
        'this,
        unsafe extern "C" fn(
            usize, // call_type
            usize, // slot_a
            usize, // slot_b
            usize, // slot_c
            usize, // slot_d
            // response_cb
            Option<
                unsafe extern "C" fn(
                    *mut libc::c_void, // response_usr
                    usize,             // response_type
                    usize,             // slot_a
                    usize,             // slot_b
                    usize,             // slot_c
                    usize,             // slot_d
                ),
            >,
            *mut libc::c_void, // response_usr
        ),
    >,
}

impl LibInner {
    unsafe fn priv_new() -> Self {
        use std::io::Write;
        let mut file =
            tempfile::NamedTempFile::new().expect("failed to open temp file");

        // TODO set some perms?

        file.write_all(LIB_BYTES)
            .expect("failed to write shared bytes");
        file.flush().expect("failed to flush shared bytes");

        // TODO set readonly?

        // TODO - keep file open as a security mitigation?
        let temp_path = file.into_temp_path();

        let lib = libloading::Library::new(&temp_path)
            .expect("failed to load shared");

        LibInnerBuilder {
            _temp_path: temp_path,
            lib,
            on_event_builder: |lib: &libloading::Library| {
                lib.get(b"OnEvent").expect("failed to load symbol")
            },
            call_builder: |lib: &libloading::Library| {
                lib.get(b"Call").expect("failed to load symbol")
            },
        }
        .build()
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: usize,
    pub error: String,
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error { code: 0, error: s }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub type CallType = usize;
pub type ResponseUsr = *mut libc::c_void;
pub type ResponseType = usize;
pub type SlotA = usize;
pub type SlotB = usize;
pub type SlotC = usize;
pub type SlotD = usize;
pub type BufferId = usize;
pub type PeerConId = usize;

pub struct Api(LibInner);

impl Api {
    fn priv_new() -> Self {
        Self(unsafe { LibInner::priv_new() })
    }

    #[inline]
    pub unsafe fn on_event<Cb>(&self, cb: Cb)
    where
        Cb: Fn(ResponseType, SlotA, SlotB, SlotC, SlotD)
            + 'static
            + Send
            + Sync,
    {
        type DynCb = Box<
            Arc<
                dyn Fn(ResponseType, SlotA, SlotB, SlotC, SlotD)
                    + 'static
                    + Send
                    + Sync,
            >,
        >;

        unsafe extern "C" fn on_event_cb(
            event_usr: *mut libc::c_void,
            event_type: ResponseType,
            slot_a: SlotA,
            slot_b: SlotB,
            slot_c: SlotC,
            slot_d: SlotD,
        ) {
            let closure: DynCb = Box::from_raw(event_usr as *mut _);

            closure(event_type, slot_a, slot_b, slot_c, slot_d);

            // need to forget it every time, otherwise drop will run
            Box::into_raw(closure);
        }

        let cb: DynCb = Box::new(Arc::new(cb));
        let cb = Box::into_raw(cb);

        let prev_usr =
            self.0.borrow_on_event()(Some(on_event_cb), cb as *mut _);

        if !prev_usr.is_null() {
            let closure: DynCb = Box::from_raw(prev_usr as *mut _);
            // *this* one we want to drop
            drop(closure);
        }
    }

    /// If the response slots are pointers to go memory, they are only valid
    /// for the duration of the callback, so make sure you know what you
    /// are doing if using this function directly.
    /// If the call slots are pointers to rust memory, go will not access them
    /// outside this call invocation.
    #[inline]
    pub unsafe fn call<Cb, R>(
        &self,
        call_type: CallType,
        slot_a: SlotA,
        slot_b: SlotB,
        slot_c: SlotC,
        slot_d: SlotD,
        cb: Cb,
    ) -> Result<R>
    where
        Cb: FnOnce(
            Result<(ResponseType, SlotA, SlotB, SlotC, SlotD)>,
        ) -> Result<R>,
    {
        let mut out = Err("not called".to_string().into());
        self.call_inner(
            call_type,
            slot_a,
            slot_b,
            slot_c,
            slot_d,
            |t, a, b, c, d| {
                out = if t == Ty::Err as usize {
                    let err = std::slice::from_raw_parts(b as *const u8, c);
                    let err = Error {
                        code: a,
                        error: String::from_utf8_lossy(err).to_string(),
                    };
                    cb(Err(err))
                } else {
                    cb(Ok((t, a, b, c, d)))
                };
            },
        );
        out
    }

    #[inline]
    unsafe fn call_inner<'lt, 'a, Cb>(
        &'lt self,
        call_type: CallType,
        slot_a: SlotA,
        slot_b: SlotB,
        slot_c: SlotC,
        slot_d: SlotD,
        cb: Cb,
    ) where
        Cb: 'a + FnOnce(ResponseType, SlotA, SlotB, SlotC, SlotD),
    {
        type DynCb<'a> =
            Box<Box<dyn FnOnce(ResponseType, SlotA, SlotB, SlotC, SlotD) + 'a>>;

        unsafe extern "C" fn call_cb(
            response_usr: *mut libc::c_void,
            response_type: ResponseType,
            slot_a: SlotA,
            slot_b: SlotB,
            slot_c: SlotC,
            slot_d: SlotD,
        ) {
            let closure: DynCb = Box::from_raw(response_usr as *mut _);

            closure(response_type, slot_a, slot_b, slot_c, slot_d);
        }

        let cb: DynCb<'a> = Box::new(Box::new(cb));
        let cb = Box::into_raw(cb);

        self.0.borrow_call()(
            call_type,
            slot_a,
            slot_b,
            slot_c,
            slot_d,
            Some(call_cb),
            cb as *mut _,
        );
    }

    /// Create a new buffer in go memory with given length,
    /// access the buffer's memory in the callback.
    #[inline]
    pub unsafe fn buffer_alloc<Cb, R>(&self, len: usize, cb: Cb) -> Result<R>
    where
        Cb: FnOnce(Result<(BufferId, &mut [u8])>) -> Result<R>,
    {
        self.call(Ty::BufferAlloc as usize, len, 0, 0, 0, move |r| match r {
            Ok((_t, a, b, c, _d)) => {
                let s = std::slice::from_raw_parts_mut(b as *mut _, c);
                cb(Ok((a, s)))
            }
            Err(e) => cb(Err(e)),
        })
    }

    #[inline]
    pub unsafe fn buffer_free(&self, id: BufferId) {
        self.0.borrow_call()(
            Ty::BufferFree as usize,
            id,
            0,
            0,
            0,
            None,
            std::ptr::null_mut(),
        );
    }

    #[inline]
    pub unsafe fn buffer_access<Cb, R>(&self, id: BufferId, cb: Cb) -> Result<R>
    where
        Cb: FnOnce(Result<(BufferId, &mut [u8])>) -> Result<R>,
    {
        self.call(Ty::BufferAccess as usize, id, 0, 0, 0, move |r| match r {
            Ok((_t, a, b, c, _d)) => {
                let s = std::slice::from_raw_parts_mut(b as *mut _, c);
                cb(Ok((a, s)))
            }
            Err(e) => cb(Err(e)),
        })
    }

    #[inline]
    pub unsafe fn peer_con_alloc(&self, json: &str) -> Result<PeerConId> {
        let len = json.as_bytes().len();
        let data = json.as_bytes().as_ptr() as usize;
        self.call(Ty::PeerConAlloc as usize, data, len, 0, 0, |r| match r {
            Ok((_t, a, _b, _c, _d)) => Ok(a),
            Err(e) => Err(e),
        })
    }

    #[inline]
    pub unsafe fn peer_con_free(&self, id: PeerConId) {
        self.0.borrow_call()(
            Ty::PeerConFree as usize,
            id,
            0,
            0,
            0,
            None,
            std::ptr::null_mut(),
        );
    }
}

/// The main entrypoint for working with this ffi binding crate.
pub static API: Lazy<Api> = Lazy::new(Api::priv_new);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buffer() {
        unsafe {
            let buf_id = API
                .buffer_alloc(8, |r| {
                    let (id, buf) = r.unwrap();
                    buf[1] = 1;
                    buf[2] = 254;
                    println!("GOT BUF: {:?}", buf);
                    Ok(id)
                })
                .unwrap();
            API.buffer_access(buf_id, |r| {
                let (_, buf) = r.unwrap();
                assert_eq!(buf[0], 0);
                assert_eq!(buf[1], 1);
                assert_eq!(buf[2], 254);
                <Result<()>>::Ok(())
            })
            .unwrap();
            API.buffer_free(buf_id);
        }
    }

    #[test]
    fn peer_con() {
        unsafe {
            let peer_con_id = API.peer_con_alloc("{}").unwrap();
            API.peer_con_free(peer_con_id);
        }
    }
}
