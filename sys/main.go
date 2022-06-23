package main

/*
#include <stdint.h>
#include <stdlib.h>

static inline uintptr_t void_star_to_ptr_t(void *ptr) {
  return (uintptr_t)ptr;
}

static inline void *ptr_to_void_star(uintptr_t ptr) {
  return (void *)ptr;
}

typedef void (*MessageCb) (
  void *usr,
  uintptr_t message_type,
  uintptr_t slot_a,
  uintptr_t slot_b,
  uintptr_t slot_c,
  uintptr_t slot_d
);

static inline void MessageCbInvoke(
  MessageCb cb,
  void *usr,
  uintptr_t message_type,
  uintptr_t slot_a,
  uintptr_t slot_b,
  uintptr_t slot_c,
  uintptr_t slot_d
) {
  cb(usr, message_type, slot_a, slot_b, slot_c, slot_d);
}
*/
import "C"

import (
  "unsafe"
  "fmt"
)

type UintPtrT = C.uintptr_t;
type MessageCb = C.MessageCb;

func VoidStarToPtrT(v unsafe.Pointer) UintPtrT {
  return C.void_star_to_ptr_t(v)
}

func PtrToVoidStar(ptr UintPtrT) unsafe.Pointer {
  return C.ptr_to_void_star(ptr)
}

func PtrToCharStar(ptr UintPtrT) *byte {
  return (*byte)(C.ptr_to_void_star(ptr))
}

func MessageCbInvoke(
  cb MessageCb,
  usr unsafe.Pointer,
  message_type UintPtrT,
  slot_a UintPtrT,
  slot_b UintPtrT,
  slot_c UintPtrT,
  slot_d UintPtrT,
) {
  C.MessageCbInvoke(cb, usr, message_type, slot_a, slot_b, slot_c, slot_d)
}

// register the MessageCb that will be invoked for events
//export OnEvent
func OnEvent(
  // the callback to invoke
  cb C.MessageCb,

  // the user data to forward to the callback
  usr unsafe.Pointer,
) {
}

// make a call into the library
//export Call
func Call(
  // call type indicator
  call_type UintPtrT,

  // input params
  slot_a UintPtrT,
  slot_b UintPtrT,
  slot_c UintPtrT,
  slot_d UintPtrT,

  // immediate result callback / response to this call
  // this will always be invoked. nil will panic
  response_cb C.MessageCb,

  // the user data to forward to the result callback
  response_usr unsafe.Pointer,
) {
  // on panics, return error text
  defer func() {
    if err := recover(); err != nil {
      if response_cb == nil {
        return
      }

      bytes := ([]byte)(fmt.Sprintf("%s", err))
      C.MessageCbInvoke(
        response_cb,
        response_usr,
        TyErr,
        // error code
        0,
        // error text ptr
        C.void_star_to_ptr_t(unsafe.Pointer(&bytes[0])),
        // error text len
        UintPtrT(len(bytes)),
        // unused
        0,
      )
    }
  }()

  // now that our panic handler is set up, delegate to callInner
  callInner(call_type, slot_a, slot_b, slot_c, slot_d, response_cb, response_usr);
}

func callInner(
  // call type indicator
  call_type UintPtrT,

  // input params
  slot_a UintPtrT,
  slot_b UintPtrT,
  slot_c UintPtrT,
  slot_d UintPtrT,

  // immediate result callback / response to this call
  // this will always be invoked. nil will panic
  response_cb C.MessageCb,

  // the user data to forward to the result callback
  response_usr unsafe.Pointer,
) {
  // -- these calls can be made even if there is no callback pointer -- //

  switch call_type {
  case TyBufferFree:
    CallBufferFree(slot_a)
    return
  case TyPeerConFree:
    CallPeerConFree(slot_a)
    return
  }

  // -- the remaining calls require response_cb to be non-nil -- //

  if response_cb == nil {
    panic(fmt.Errorf("response_cb cannot be nil for call_type: %d", call_type))
  }

  switch call_type {
  case TyBufferAlloc:
    CallBufferAlloc(slot_a, response_cb, response_usr)
  case TyBufferAccess:
    CallBufferAccess(slot_a, response_cb, response_usr)
  case TyPeerConAlloc:
    CallPeerConAlloc(slot_a, slot_b, response_cb, response_usr)
  default:
    panic(fmt.Errorf("invalid call_type: %d", call_type))
  }
}

func main() {}
