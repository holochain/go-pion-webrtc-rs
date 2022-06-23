package main

import (
  "runtime/cgo"
  "unsafe"
  "encoding/json"
  "sync"

  "github.com/pion/webrtc/v3"
)

type PeerCon struct {
  mu sync.Mutex
  closed bool
  con *webrtc.PeerConnection
  handle UintPtrT
}

func (peer_con *PeerCon) Free() {
  peer_con.mu.Lock()
  defer peer_con.mu.Unlock()

  if peer_con.closed {
    return
  }

  peer_con.closed = true
  (cgo.Handle)(peer_con.handle).Delete()

  peer_con.con.Close()
  peer_con.con = nil
}

func CallPeerConAlloc(
  config_json UintPtrT,
  config_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  bytes := unsafe.Slice(PtrToCharStar(config_json), config_len)

  var config_parsed webrtc.Configuration
  if err := json.Unmarshal(bytes, &config_parsed); err != nil {
    panic(err)
  }

  con, err := webrtc.NewPeerConnection(config_parsed)
  if err != nil {
    panic(err)
  }

  peer_con := new(PeerCon)
  peer_con.con = con
  peer_con.handle = UintPtrT(cgo.NewHandle(peer_con))
  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConAlloc,
    peer_con.handle,
    0,
    0,
    0,
  )
}

func CallPeerConFree(id UintPtrT) {
  hnd := cgo.Handle(id)
  peer_con := hnd.Value().(*PeerCon)
  peer_con.Free()
}
