import ctypes
from ctypes import wintypes
import time

kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

GENERIC_READ = 0x80000000
GENERIC_WRITE = 0x40000000
OPEN_EXISTING = 3
FILE_ATTRIBUTE_NORMAL = 0x80

CreateFileW = kernel32.CreateFileW
CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
CreateFileW.restype = wintypes.HANDLE

DeviceIoControl = kernel32.DeviceIoControl
DeviceIoControl.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, wintypes.LPVOID, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD), wintypes.LPVOID]
DeviceIoControl.restype = wintypes.BOOL

h_device = CreateFileW(r"\\.\ReadAndWritePort", GENERIC_READ | GENERIC_WRITE, 0, None, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, None)

def write_port(port, data):
    in_buf = (ctypes.c_uint16 * 2)(port, data)
    ret = wintypes.DWORD(0)
    DeviceIoControl(h_device, 0x9C402008, in_buf, 4, None, 0, ctypes.byref(ret), None)

def read_port(port):
    in_buf = (ctypes.c_uint16 * 1)(port)
    out_buf = (ctypes.c_uint8 * 1)(0)
    ret = wintypes.DWORD(0)
    DeviceIoControl(h_device, 0x9C402004, in_buf, 2, out_buf, 1, ctypes.byref(ret), None)
    return out_buf[0]

# Let's test reading CPU Fan
write_port(0x6C, 0xD5)
time.sleep(0.01)
write_port(0x68, 24)
time.sleep(0.01)
cpu_low = read_port(0x68)

time.sleep(0.01)
write_port(0x6C, 0xD5)
time.sleep(0.01)
write_port(0x68, 25)
time.sleep(0.01)
cpu_high = read_port(0x68)

cpu_fan = (cpu_high << 8) | cpu_low
print(f"CPU Fan: high={cpu_high} (0x{cpu_high:02X}), low={cpu_low} (0x{cpu_low:02X}), RPM={cpu_fan}")

# GPU Fan
write_port(0x6C, 0xD5)
time.sleep(0.01)
write_port(0x68, 22)
time.sleep(0.01)
gpu_low = read_port(0x68)

time.sleep(0.01)
write_port(0x6C, 0xD5)
time.sleep(0.01)
write_port(0x68, 23)
time.sleep(0.01)
gpu_high = read_port(0x68)

gpu_fan = (gpu_high << 8) | gpu_low
print(f"GPU Fan: high={gpu_high} (0x{gpu_high:02X}), low={gpu_low} (0x{gpu_low:02X}), RPM={gpu_fan}")

# CPU Temp
write_port(0x6C, 0xDD)
time.sleep(0.01)
write_port(0x68, 32)
time.sleep(0.01)
cpu_temp = read_port(0x68)
print(f"CPU Temp (cmd 32): {cpu_temp} (0x{cpu_temp:02X})")

# GPU Temp
write_port(0x6C, 0xDD)
time.sleep(0.01)
write_port(0x68, 35)
time.sleep(0.01)
gpu_temp = read_port(0x68)
print(f"GPU Temp (cmd 35): {gpu_temp} (0x{gpu_temp:02X})")

# What if cmd 33, 34?
for cmd in [30, 31, 32, 33, 34, 35, 36, 37]:
    write_port(0x6C, 0xDD)
    time.sleep(0.01)
    write_port(0x68, cmd)
    time.sleep(0.01)
    val = read_port(0x68)
    print(f"Cmd {cmd} -> {val} (0x{val:02X})")

kernel32.CloseHandle(h_device)
