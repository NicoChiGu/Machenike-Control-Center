import ctypes
from ctypes import wintypes
import time

kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

h_device = kernel32.CreateFileW(r"\\.\ReadAndWritePort", 0xC0000000, 0, None, 3, 0x80, None)

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

def get_cpu_temp():
    val = 0
    for _ in range(10):
        write_port(0x6C, 0xDD)
        time.sleep(0.01)
        write_port(0x68, 32)
        time.sleep(0.01)
        val = read_port(0x68)
        if 10 <= val <= 120:
            break
        time.sleep(0.01)
    return val

def get_gpu_temp():
    val = 0
    for _ in range(10):
        write_port(0x6C, 0xDD)
        time.sleep(0.01)
        write_port(0x68, 35)
        time.sleep(0.01)
        val = read_port(0x68)
        if 10 <= val <= 120:
            break
        time.sleep(0.01)
    return val

def get_gpu_fan():
    write_port(0x6C, 0xD5)
    time.sleep(0.01)
    write_port(0x68, 22)
    time.sleep(0.01)
    low = read_port(0x68)
    time.sleep(0.01)
    write_port(0x6C, 0xD5)
    time.sleep(0.01)
    write_port(0x68, 23)
    time.sleep(0.01)
    high = read_port(0x68)
    return (high << 8) | low

def get_cpu_fan():
    write_port(0x6C, 0xD5)
    time.sleep(0.01)
    write_port(0x68, 24)
    time.sleep(0.01)
    low = read_port(0x68)
    time.sleep(0.01)
    write_port(0x6C, 0xD5)
    time.sleep(0.01)
    write_port(0x68, 25)
    time.sleep(0.01)
    high = read_port(0x68)
    return (high << 8) | low

for i in range(5):
    ct = get_cpu_temp()
    gt = get_gpu_temp()
    cf = get_cpu_fan()
    gf = get_gpu_fan()
    print(f"[{i}] CPU Temp: {ct} °C, GPU Temp: {gt} °C, CPU Fan: {cf} RPM, GPU Fan: {gf} RPM")
    time.sleep(0.5)

kernel32.CloseHandle(h_device)
