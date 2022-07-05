package main

import (
  "bytes"
	"runtime/cgo"
	"sync"
	"unsafe"
)

type Buffer struct {
	mu     sync.Mutex
	closed bool
  buf bytes.Buffer
	handle UintPtrT
}

// If you invoke this function, you *must* call Free,
// otherwise the buffer will be leaked.
func NewBuffer() *Buffer {
	buf := new(Buffer)
	buf.handle = UintPtrT(cgo.NewHandle(buf))
	return buf
}

func (buf *Buffer) Free() {
	buf.mu.Lock()
	defer buf.mu.Unlock()

	if buf.closed {
		return
	}

	buf.closed = true
	(cgo.Handle)(buf.handle).Delete()
}

func CallBufferAlloc(
	response_cb MessageCb,
	response_usr unsafe.Pointer,
) {
	buf := NewBuffer()
	MessageCbInvoke(
		response_cb,
		response_usr,
		TyBufferAlloc,
		buf.handle,
		0,
		0,
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

  bytes := buf.buf.Bytes()

  if bytes == nil {
    MessageCbInvoke(
      response_cb,
      response_usr,
      TyBufferAccess,
      buf.handle,
      0,
      0,
      0,
    )
    return
  }

	MessageCbInvoke(
		response_cb,
		response_usr,
		TyBufferAccess,
		buf.handle,
		VoidStarToPtrT(unsafe.Pointer(&(bytes)[0])),
		UintPtrT(len(bytes)),
		0,
	)
}
