package main

import (
	"fmt"
	"runtime/cgo"
	"sync"
	"unsafe"

	"github.com/pion/webrtc/v3"
)

type DataChan struct {
	mu     sync.Mutex
	closed bool
	ch     *webrtc.DataChannel
	handle UintPtrT
}

// If you invoke this function, you *must* call Free,
// otherwise the channel will be leaked.
func NewDataChan(ch *webrtc.DataChannel) *DataChan {
	dataChan := new(DataChan)
	dataChan.ch = ch
	dataChan.handle = UintPtrT(cgo.NewHandle(dataChan))

	ch.OnClose(func() {
		fmt.Printf("go DATA CHAN CLOSE\n")
	})

	ch.OnOpen(func() {
		fmt.Printf("go DATA CHAN OPEN\n")
	})

	ch.OnError(func(err error) {
		fmt.Printf("go DATA CHAN ERR %v\n", err)
	})

	ch.OnMessage(func(msg webrtc.DataChannelMessage) {
		fmt.Printf("go DATA CHAN MSG %v\n", msg)
	})

	return dataChan
}

func (dataChan *DataChan) Free() {
	dataChan.mu.Lock()
	defer dataChan.mu.Unlock()

	if dataChan.closed {
		return
	}

	dataChan.closed = true
	(cgo.Handle)(dataChan.handle).Delete()

	dataChan.ch.Close()
	dataChan.ch = nil
}

func CallDataChanFree(data_chan_id UintPtrT) {
	hnd := cgo.Handle(data_chan_id)
	dataChan := hnd.Value().(*DataChan)
	dataChan.Free()
}

func CallDataChanSend(
	data_chan_id UintPtrT,
	buffer_id UintPtrT,
	response_cb MessageCb,
	response_usr unsafe.Pointer,
) {
	hnd := cgo.Handle(data_chan_id)
	dataChan := hnd.Value().(*DataChan)
	dataChan.mu.Lock()
	defer dataChan.mu.Unlock()

	if dataChan.closed {
		panic("DataChanClosed")
	}

	buf_hnd := cgo.Handle(buffer_id)
	buf := buf_hnd.Value().(*Buffer)
	buf.mu.Lock()
	defer buf.mu.Unlock()

	if buf.closed {
		panic("BufferClosed")
	}

	panic("TODO")
}
