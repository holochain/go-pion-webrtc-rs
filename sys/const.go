package main

const (
  // Reporting an error from the lib.
  // - allowed contexts: Event, Response
  // - msg slot_a = error code
  // - msg slot_b = utf8 error text ptr
  // - msg slot_c = utf8 error text len
  TyErr UintPtrT = 0xffff

  // Request a go buffer be created / giving access to said buffer in resp.
  // - allowed contexts: Call, Response
  // - call slot_a: buffer size
  // - msg slot_a: buffer id
  // - msg slot_b: buffer ptr
  // - msg slot_c: buffer len
  TyBufferAlloc UintPtrT = 0x8001

  // Request an existing buffer be released. It will no longer be accessible.
  // - allowed contexts: Call, Response
  // - call slot_a: buffer id
  TyBufferFree UintPtrT = 0x8002

  // Request access to an existing buffer.
  // - allowed contexts: Call, Response
  // - call slot_a: buffer id
  // - msg slot_a: buffer id
  // - msg slot_b: buffer ptr
  // - msg slot_c: buffer len
  TyBufferAccess UintPtrT = 0x8003

  // Request a new peer connection be opened.
  // - allowed contexts: Call, Response
  // - call slot_a: utf8 json config ptr
  // - call slot_b: utf8 json config len
  // - msg slot_a: peer_con id
  TyPeerConAlloc UintPtrT = 0x9001

  // Request an existing peer con be closed and released.
  // - allowed contexts: Call, Response
  // - call slot_a: peer_con id
  TyPeerConFree UintPtrT = 0x9002

  // OnICECandidate event on an existing peer con.
  // - allowed contexts: Event
  // - msg slot_a: peer_con id
  // - msg slot_b: utf8 json ptr
  // - msg slot_c: utf8 json len
  TyPeerConICECandidate UintPtrT = 0x9801
)
