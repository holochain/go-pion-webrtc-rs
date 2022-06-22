//! Access the go-pion-webrtc api interface using the
//! pub once_cell::sync::Lazy static static [API] handle.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

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
        libloading::Symbol<'this, unsafe extern "C" fn(libc::c_int) -> usize>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_free: libloading::Symbol<'this, unsafe extern "C" fn(usize)>,
    #[borrows(lib)]
    #[not_covariant]
    go_slice_len:
        libloading::Symbol<'this, unsafe extern "C" fn(usize) -> libc::c_int>,
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
                    libc::c_int,
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

        // TODO set some perms?

        file.write(LIB_BYTES).expect("failed to write shared bytes");
        file.flush().expect("failed to flush shared bytes");

        // TODO set readonly?

        // TODO - keep file open as a security mitigation?
        let temp_path = file.into_temp_path();

        let lib = libloading::Library::new(&temp_path)
            .expect("failed to load shared");

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

pub struct Api(LibInner);

impl Api {
    fn priv_new() -> Self {
        Self(unsafe { LibInner::priv_new() })
    }

    #[inline(always)]
    pub unsafe fn c_hello(&self) -> *mut libc::c_char {
        self.0.with_c_hello(|f| f())
    }

    #[inline(always)]
    pub unsafe fn go_slice_alloc(&self, length: libc::c_int) -> usize {
        self.0.with_go_slice_alloc(|f| f(length))
    }

    #[inline(always)]
    pub unsafe fn go_slice_free(&self, slice_hnd: usize) {
        self.0.with_go_slice_free(|f| f(slice_hnd));
    }

    #[inline(always)]
    pub unsafe fn go_slice_len(&self, slice_hnd: usize) -> libc::c_int {
        self.0.with_go_slice_len(|f| f(slice_hnd))
    }

    #[inline(always)]
    pub unsafe fn go_slice_read(
        &self,
        slice_hnd: usize,
        usr: *mut libc::c_void,
        cb: unsafe extern "C" fn(
            *mut libc::c_void,
            libc::c_int,
            *const libc::c_char,
        ),
    ) {
        self.0.with_go_slice_read(|f| f(slice_hnd, usr, Some(cb)));
    }
}

/// The main entrypoint for working with this ffi binding crate.
pub static API: Lazy<Api> = Lazy::new(|| Api::priv_new());

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_c_hello() {
        let c_str = unsafe {
            let c_str = std::ffi::CStr::from_ptr(API.c_hello());
            c_str.to_str().unwrap().to_owned()
        };
        println!("got: {}", c_str);
        assert_eq!("video/H264", c_str.as_str());
    }

    #[inline(always)]
    unsafe fn call_go_slice_read<F>(slice_hnd: usize, cb: F)
    where
        F: FnOnce(&[u8]),
    {
        #[inline(always)]
        unsafe extern "C" fn callback_delegate(
            usr: *mut libc::c_void,
            len: libc::c_int,
            data: *const libc::c_char,
        ) {
            let closure: Box<Box<dyn FnOnce(&[u8])>> =
                Box::from_raw(usr as *mut _);
            closure(std::slice::from_raw_parts(
                data as *const u8,
                len as usize,
            ));
        }

        // double box, otherwise it's a fat pointer
        let cb: Box<Box<dyn FnOnce(&[u8])>> = Box::new(Box::new(cb));
        let cb = Box::into_raw(cb);

        API.go_slice_read(slice_hnd, cb as *mut _, callback_delegate);
    }

    #[test]
    fn go_slice() {
        let slice = unsafe { API.go_slice_alloc(8) };
        let len = unsafe { API.go_slice_len(slice) };
        assert_eq!(8, len);
        unsafe {
            call_go_slice_read(slice, |data| {
                println!("{:?}", data);
            });
        }
        unsafe { API.go_slice_free(slice) };
    }
}
