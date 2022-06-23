package main

import (
  "runtime/cgo"
  "unsafe"
  "sync"
)

type Buffer struct {
  mu sync.Mutex
  closed bool
  data *[]byte
  handle UintPtrT
}

// If you invoke this function, you *must* call Free,
// otherwise the buffer will be leaked.
func NewBuffer(length UintPtrT) *Buffer {
  buf := new(Buffer)
  tmp := make([]byte, length)
  buf.data = &tmp
  buf.handle = UintPtrT(cgo.NewHandle(buf))
  return buf
}

func (buf *Buffer) Free() *[]byte {
  buf.mu.Lock()
  defer buf.mu.Unlock()

  if buf.closed {
    return nil
  }

  buf.closed = true
  (cgo.Handle)(buf.handle).Delete()

  out := buf.data
  buf.data = nil

  return out
}

func CallBufferAlloc(
  length UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  buf := NewBuffer(length)
  MessageCbInvoke(
    response_cb,
    response_usr,
    TyBufferAlloc,
    buf.handle,
    VoidStarToPtrT(unsafe.Pointer(&(*buf.data)[0])),
    UintPtrT(len(*buf.data)),
    0,
  )
}

func CallBufferFree(id UintPtrT) {
  hnd := cgo.Handle(id)
  buf := hnd.Value().(*Buffer)
  buf.Free()
}

func CallBufferAccess(
  id UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(id)
  buf := hnd.Value().(*Buffer)
  buf.mu.Lock()
  defer buf.mu.Unlock()

  if buf.closed {
    panic("BufferClosed")
  }

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyBufferAccess,
    buf.handle,
    VoidStarToPtrT(unsafe.Pointer(&(*buf.data)[0])),
    UintPtrT(len(*buf.data)),
    0,
  )
}
