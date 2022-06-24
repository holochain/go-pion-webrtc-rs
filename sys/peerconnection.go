package main

import (
  "runtime/cgo"
  "unsafe"
  "encoding/json"
  "sync"
  "fmt"

  "github.com/pion/webrtc/v3"
)

type PeerCon struct {
  mu sync.Mutex
  closed bool
  con *webrtc.PeerConnection
  dataChan *webrtc.DataChannel
  handle UintPtrT
}

func (peerCon *PeerCon) Free() {
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  peerCon.closed = true
  (cgo.Handle)(peerCon.handle).Delete()

  peerCon.con.Close()
  peerCon.con = nil
}

func CallPeerConAlloc(
  config_json UintPtrT,
  config_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  buf := LoadBytesSafe(config_json, config_len)

  var config_parsed webrtc.Configuration
  if err := json.Unmarshal(buf.Bytes(), &config_parsed); err != nil {
    panic(err)
  }

  con, err := webrtc.NewPeerConnection(config_parsed)
  if err != nil {
    panic(err)
  }

  peerCon := new(PeerCon)
  peerCon.con = con

  handle := UintPtrT(cgo.NewHandle(peerCon))
  peerCon.handle = handle

  con.OnICECandidate(func(candidate *webrtc.ICECandidate) {
    if candidate == nil {
      return
    }

    json, err := json.Marshal(candidate.ToJSON())
    if err != nil {
      return
    }
    bytes := []byte(json)

    EmitEvent(
      TyPeerConOnICECandidate,
      handle,
      VoidStarToPtrT(unsafe.Pointer(&bytes[0])),
      UintPtrT(len(bytes)),
      0,
    )
  })

  con.OnConnectionStateChange(func(state webrtc.PeerConnectionState) {
    EmitEvent(
      TyPeerConOnStateChange,
      handle,
      UintPtrT(state),
      0,
      0,
    )
  })

  con.OnDataChannel(func(dataChan *webrtc.DataChannel) {
    fmt.Printf("go GOT DATA CHANNEL\n");
  })

  // -- rm
  dataChan, err := peerCon.con.CreateDataChannel("data", nil)
  if err != nil {
    panic(err)
  }
  peerCon.dataChan = dataChan
  // -- end rm

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConAlloc,
    peerCon.handle,
    0,
    0,
    0,
  )
}

func CallPeerConFree(peer_con_id UintPtrT) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.Free()
}

func CallPeerConCreateOffer(
  peer_con_id UintPtrT,
  json_data UintPtrT,
  json_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  var opts *webrtc.OfferOptions

  if json_data != 0 {
    buf := LoadBytesSafe(json_data, json_len)

    opts = new(webrtc.OfferOptions)
    if err := json.Unmarshal(buf.Bytes(), opts); err != nil {
      panic(err)
    }
  }

  offer, err := peerCon.con.CreateOffer(opts)
  if err != nil {
    panic(err)
  }

  offerJson, err := json.Marshal(offer)
  if err != nil {
    panic(err)
  }

  offerBytes := []byte(offerJson)

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConCreateOffer,
    VoidStarToPtrT(unsafe.Pointer(&offerBytes[0])),
    UintPtrT(len(offerBytes)),
    0,
    0,
  )
}

func CallPeerConCreateAnswer(
  peer_con_id UintPtrT,
  json_data UintPtrT,
  json_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  var opts *webrtc.AnswerOptions

  if json_data != 0 {
    buf := LoadBytesSafe(json_data, json_len)

    opts = new(webrtc.AnswerOptions)
    if err := json.Unmarshal(buf.Bytes(), opts); err != nil {
      panic(err)
    }
  }

  offer, err := peerCon.con.CreateAnswer(opts)
  if err != nil {
    panic(err)
  }

  offerJson, err := json.Marshal(offer)
  if err != nil {
    panic(err)
  }

  offerBytes := []byte(offerJson)

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConCreateAnswer,
    VoidStarToPtrT(unsafe.Pointer(&offerBytes[0])),
    UintPtrT(len(offerBytes)),
    0,
    0,
  )
}

func CallPeerConSetLocalDesc(
  peer_con_id UintPtrT,
  json_data UintPtrT,
  json_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  buf := LoadBytesSafe(json_data, json_len)

  var desc webrtc.SessionDescription;
  if err := json.Unmarshal(buf.Bytes(), &desc); err != nil {
    panic(err)
  }

  if err := peerCon.con.SetLocalDescription(desc); err != nil {
    panic(err)
  }

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConSetLocalDesc,
    0,
    0,
    0,
    0,
  )
}

func CallPeerConSetRemDesc(
  peer_con_id UintPtrT,
  json_data UintPtrT,
  json_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  buf := LoadBytesSafe(json_data, json_len)

  var desc webrtc.SessionDescription;
  if err := json.Unmarshal(buf.Bytes(), &desc); err != nil {
    panic(err)
  }

  if err := peerCon.con.SetRemoteDescription(desc); err != nil {
    panic(err)
  }

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConSetRemDesc,
    0,
    0,
    0,
    0,
  )
}

func CallPeerConAddICECandidate(
  peer_con_id UintPtrT,
  json_data UintPtrT,
  json_len UintPtrT,
  response_cb MessageCb,
  response_usr unsafe.Pointer,
) {
  hnd := cgo.Handle(peer_con_id)
  peerCon := hnd.Value().(*PeerCon)
  peerCon.mu.Lock()
  defer peerCon.mu.Unlock()

  if peerCon.closed {
    return
  }

  buf := LoadBytesSafe(json_data, json_len)

  var candidate webrtc.ICECandidateInit;
  if err := json.Unmarshal(buf.Bytes(), &candidate); err != nil {
    panic(err)
  }

  if err := peerCon.con.AddICECandidate(candidate); err != nil {
    panic(err)
  }

  MessageCbInvoke(
    response_cb,
    response_usr,
    TyPeerConAddICECandidate,
    0,
    0,
    0,
    0,
  )
}
