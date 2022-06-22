package main

/*
#include <stdint.h>
typedef void (*GoSliceReadCb) (void *usr, int len, const char *data);
static inline void GoSliceReadCbInvoke(GoSliceReadCb cb, void *usr, int len, const char *data) {
  cb(usr, len, data);
}
*/
import "C"

import (
  "runtime/cgo"
  "unsafe"
  "fmt"

  //"github.com/pion/datachannel"
  "github.com/pion/webrtc/v3"
)

func Hello() string {
  //return "hello world!"
  return webrtc.MimeTypeH264
}

//export GoSliceAlloc
func GoSliceAlloc(length C.int) C.uintptr_t {
  slice := make([]byte, length)
  slice[0] = 0
  slice[1] = 255
  slice[2] = 127
  slice[3] = 128
  slice[4] = 129
  return C.uintptr_t(cgo.NewHandle(slice))
}

//export GoSliceFree
func GoSliceFree(slice_hnd C.uintptr_t) {
  hnd := cgo.Handle(slice_hnd)
  hnd.Delete()
}

//export GoSliceLen
func GoSliceLen(slice_hnd C.uintptr_t) C.int {
  hnd := cgo.Handle(slice_hnd)
  slice := hnd.Value().([]byte)
  return C.int(len(slice))
}

//export GoSliceRead
func GoSliceRead(slice_hnd C.uintptr_t, usr unsafe.Pointer, cb C.GoSliceReadCb) {
  hnd := cgo.Handle(slice_hnd)
  slice := hnd.Value().([]byte)
  C.GoSliceReadCbInvoke(
    cb,
    usr,
    C.int(len(slice)),
    (*C.char)(unsafe.Pointer(&slice[0])),
  )
}

//export CHello
func CHello() *C.char {
  return C.CString(fmt.Sprintf("%v", Hello()))
}

func main() {}
