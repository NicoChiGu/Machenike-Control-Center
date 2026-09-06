import ctypes
from ctypes import wintypes
import time

kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

h_device = kernel32.CreateFileW(r"\\.\ReadAndWritePort", 0xC0000000, 0, None, 3, 0x80, None)
h_mutex = kernel32.CreateMutexW(None, False, "Global\\IO_Mutex")

def lock():
    kernel32.WaitForSingleObject(h_mutex, 1000)

def unlock():
    kernel32.ReleaseMutex(h_mutex)

def write_port(port, data):
    in_buf = (ctypes.c_uint16 * 2)(port, data)
    ret = wintypes.DWORD(0)
    kernel32.DeviceIoControl(h_device, 0x9C402008, in_buf, 4, None, 0, ctypes.byref(ret), None)

def read_port(port):
    in_buf = (ctypes.c_uint16 * 1)(port)
    out_buf = (ctypes.c_uint8 * 1)(0)
    ret = wintypes.DWORD(0)
    kernel32.DeviceIoControl(h_device, 0x9C402004, in_buf, 2, out_buf, 1, ctypes.byref(ret), None)
    return out_buf[0]

def read_temp(subcmd):
    lock()
    try:
        for _ in range(5):
            write_port(0x6C, 0xDD)
            time.sleep(0.01)
            write_port(0x68, subcmd)
            time.sleep(0.01)
            val = read_port(0x68)
            if 20 <= val <= 110:
                return val
            time.sleep(0.01)
        return val
    finally:
        unlock()

def read_fan(cmd_low, cmd_high):
    lock()
    try:
        for _ in range(5):
            write_port(0x6C, 0xD5)
            time.sleep(0.01)
            write_port(0x68, cmd_low)
            time.sleep(0.01)
            low = read_port(0x68)

            time.sleep(0.01)
            write_port(0x6C, 0xD5)
            time.sleep(0.01)
            write_port(0x68, cmd_high)
            time.sleep(0.01)
            high = read_port(0x68)

            rpm = (high << 8) | low
            if 0 <= rpm <= 6500:
                return rpm
            time.sleep(0.01)
        return rpm
    finally:
        unlock()

print("Testing with transaction mutex locking:")
for i in range(5):
    ct = read_temp(32)
    gt = read_temp(35)
    cf = read_fan(24, 25)
    gf = read_fan(22, 23)
    print(f"Sample {i}: CPU Temp = {ct}°C, GPU Temp = {gt}°C, CPU Fan = {cf} RPM, GPU Fan = {gf} RPM")
    time.sleep(0.5)

kernel32.CloseHandle(h_mutex)
kernel32.CloseHandle(h_device)
