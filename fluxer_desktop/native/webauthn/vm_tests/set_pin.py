# SPDX-License-Identifier: AGPL-3.0-or-later

import ctypes
import sys

device_path = sys.argv[1].encode() if len(sys.argv) > 1 else b"/dev/hidraw4"
pin = sys.argv[2].encode() if len(sys.argv) > 2 else b"123456"

lib = ctypes.CDLL("libfido2.so.1")
lib.fido_init(0)
lib.fido_dev_new.restype = ctypes.c_void_p
lib.fido_strerr.restype = ctypes.c_char_p

dev = lib.fido_dev_new()
rc = lib.fido_dev_open(ctypes.c_void_p(dev), device_path)
if rc != 0:
    print(f"open failed rc={rc} {lib.fido_strerr(rc).decode()}")
    sys.exit(1)
rc = lib.fido_dev_set_pin(ctypes.c_void_p(dev), pin, None)
print(f"set_pin rc={rc} {lib.fido_strerr(rc).decode()}")
lib.fido_dev_close(ctypes.c_void_p(dev))
sys.exit(0 if rc == 0 else 1)
