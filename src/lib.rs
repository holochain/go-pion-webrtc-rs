//! Rather than trying to use the fn apis directly, please use
//! our pub once_cell::sync::Lazy static static [LIB] handle.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.rs"));

#[cfg(target_os = "macos")]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.dylib"));

#[cfg(target_os = "windows")]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.dll"));

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const LIB_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/go-pion-webrtc.so"));

use once_cell::sync::Lazy;

#[ouroboros::self_referencing]
struct LibInner {
    _temp_path: tempfile::TempPath,
    lib: libloading::Library,
    #[borrows(lib)]
    #[not_covariant]
    c_hello:
        libloading::Symbol<'this, unsafe extern "C" fn() -> *mut libc::c_char>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_alloc:
        libloading::Symbol<'this, unsafe extern "C" fn(GoInt) -> usize>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_free: libloading::Symbol<'this, unsafe extern "C" fn(usize)>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_len: libloading::Symbol<'this, unsafe extern "C" fn(usize) -> i64>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_read: libloading::Symbol<
        'this,
        unsafe extern "C" fn(
            usize,
            *mut libc::c_void,
            Option<
                unsafe extern "C" fn(
                    *mut libc::c_void,
                    i64,
                    *const libc::c_char,
                ),
            >,
        ),
    >,
}

impl LibInner {
    unsafe fn priv_new() -> Self {
        use std::io::Write;
        let mut file =
            tempfile::NamedTempFile::new().expect("failed to open temp file");
        file.write(LIB_BYTES).expect("failed to write shared bytes");
        file.flush().expect("failed to flush shared bytes");
        let temp_path = file.into_temp_path();

        let lib =
            libloading::Library::new(&temp_path).expect("faile to load shared");

        LibInnerBuilder {
            _temp_path: temp_path,
            lib,
            c_hello_builder: |lib: &libloading::Library| {
                lib.get(b"CHello").expect("failed to load symbol")
            },
            go_slice_alloc_builder: |lib: &libloading::Library| {
                lib.get(b"GoSliceAlloc").expect("failed to load symbol")
            },
            go_slice_free_builder: |lib: &libloading::Library| {
                lib.get(b"GoSliceFree").expect("failed to load symbol")
            },
            go_slice_len_builder: |lib: &libloading::Library| {
                lib.get(b"GoSliceLen").expect("failed to load symbol")
            },
            go_slice_read_builder: |lib: &libloading::Library| {
                lib.get(b"GoSliceRead").expect("failed to load symbol")
            },
        }
        .build()
    }
}

pub struct Lib(LibInner);

impl Lib {
    fn priv_new() -> Self {
        Self(unsafe { LibInner::priv_new() })
    }

    pub unsafe fn c_hello(&self) -> *mut libc::c_char {
        self.0.with_c_hello(|f| f())
    }

    pub unsafe fn go_slice_alloc(&self, length: GoInt) -> usize {
        self.0.with_go_slice_alloc(|f| f(length))
    }

    pub unsafe fn go_slice_free(&self, slice_hnd: usize) {
        self.0.with_go_slice_free(|f| f(slice_hnd));
    }

    pub unsafe fn go_slice_len(&self, slice_hnd: usize) -> i64 {
        self.0.with_go_slice_len(|f| f(slice_hnd))
    }

    pub unsafe fn go_slice_read(
        &self,
        slice_hnd: usize,
        usr: *mut libc::c_void,
        cb: unsafe extern "C" fn(*mut libc::c_void, i64, *const libc::c_char),
    ) {
        self.0.with_go_slice_read(|f| f(slice_hnd, usr, Some(cb)));
    }
}

/// The main entrypoint for working with this ffi binding crate.
pub static LIB: Lazy<Lib> = Lazy::new(|| Lib::priv_new());

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_c_hello() {
        let c_str = unsafe {
            let c_str = std::ffi::CStr::from_ptr(LIB.c_hello());
            c_str.to_str().unwrap().to_owned()
        };
        println!("got: {}", c_str);
    }

    #[inline(always)]
    unsafe fn call_rust_go_slice_read<F>(slice_hnd: usize, cb: F)
    where
        F: FnMut(&[u8]),
    {
        #[inline(always)]
        unsafe extern "C" fn callback_delegate(
            usr: *mut libc::c_void,
            len: i64,
            data: *const libc::c_char,
        ) {
            let closure: &mut Box<dyn FnMut(&[u8])> = std::mem::transmute(usr);
            closure(std::slice::from_raw_parts(
                data as *const u8,
                len as usize,
            ));
        }

        let cb: Box<Box<dyn FnMut(&[u8])>> = Box::new(Box::new(cb));
        let cb = Box::into_raw(cb);

        LIB.go_slice_read(slice_hnd, cb as *mut _, callback_delegate);

        drop(Box::from_raw(cb));
    }

    #[test]
    fn go_slice() {
        let slice = unsafe { LIB.go_slice_alloc(8) };
        let len = unsafe { LIB.go_slice_len(slice) };
        assert_eq!(8, len);
        unsafe {
            call_rust_go_slice_read(slice, |data| {
                println!("{:?}", data);
            });
        }
        unsafe { LIB.go_slice_free(slice) };
    }
}
