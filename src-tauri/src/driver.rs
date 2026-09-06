use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::Mutex;

// Win32 Constants
const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;
const OPEN_EXISTING: u32 = 3;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
const INVALID_HANDLE_VALUE: isize = -1;

const IOCTL_READ_PORT: u32 = 0x9C402004;
const IOCTL_WRITE_PORT: u32 = 0x9C402008;

#[repr(C)]
struct WritePortBuffer {
    port: u16,
    data: u16,
}

#[repr(C)]
struct ReadPortBuffer {
    port: u16,
}

extern "system" {
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: *mut c_void,
    ) -> isize;

    fn DeviceIoControl(
        hDevice: isize,
        dwIoControlCode: u32,
        lpInBuffer: *const c_void,
        nInBufferSize: u32,
        lpOutBuffer: *mut c_void,
        nOutBufferSize: u32,
        lpBytesReturned: *mut u32,
        lpOverlapped: *mut c_void,
    ) -> i32;

    fn CloseHandle(hObject: isize) -> i32;

    fn CreateMutexW(
        lpMutexAttributes: *mut c_void,
        bInitialOwner: i32,
        lpName: *const u16,
    ) -> isize;

    fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;

    fn ReleaseMutex(hMutex: isize) -> i32;
}

pub struct PortDriver {
    handle: isize,
    named_mutex: isize,
    local_lock: Mutex<()>,
}

unsafe impl Send for PortDriver {}
unsafe impl Sync for PortDriver {}

impl PortDriver {
    pub fn new() -> Result<Self, String> {
        let path: Vec<u16> = r"\\.\ReadAndWritePort"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };

        // If not opened, attempt to start the ioportdrv service and retry
        if handle == INVALID_HANDLE_VALUE || handle == 0 {
            let _ = std::process::Command::new("sc.exe")
                .args(["start", "ioportdrv"])
                .output();
            std::thread::sleep(std::time::Duration::from_millis(150));

            handle = unsafe {
                CreateFileW(
                    path.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    null_mut(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                )
            };
        }

        if handle == INVALID_HANDLE_VALUE || handle == 0 {
            return Err("Failed to open \\\\.\\ReadAndWritePort. Is ioportdrv service running? Please run install_driver.bat as Administrator.".into());
        }

        let mutex_name: Vec<u16> = "Global\\IO_Mutex"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let named_mutex = unsafe { CreateMutexW(null_mut(), 0, mutex_name.as_ptr()) };

        Ok(Self {
            handle,
            named_mutex,
            local_lock: Mutex::new(()),
        })
    }

    pub fn transaction<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = self.local_lock.lock().unwrap();
        if self.named_mutex != 0 && self.named_mutex != INVALID_HANDLE_VALUE {
            unsafe {
                WaitForSingleObject(self.named_mutex, 1000);
            }
        }

        let res = f();

        if self.named_mutex != 0 && self.named_mutex != INVALID_HANDLE_VALUE {
            unsafe {
                ReleaseMutex(self.named_mutex);
            }
        }

        res
    }

    pub fn write_port_raw(&self, port: u8, data: u8) -> Result<(), String> {
        let buf = WritePortBuffer {
            port: port as u16,
            data: data as u16,
        };
        let mut returned: u32 = 0;

        let ok = unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_WRITE_PORT,
                &buf as *const _ as *const c_void,
                std::mem::size_of::<WritePortBuffer>() as u32,
                null_mut(),
                0,
                &mut returned,
                null_mut(),
            )
        };

        if ok != 0 {
            Ok(())
        } else {
            Err(format!("DeviceIoControl write failed on port 0x{:02X}", port))
        }
    }

    pub fn read_port_raw(&self, port: u8) -> Result<u8, String> {
        let in_buf = ReadPortBuffer {
            port: port as u16,
        };
        let mut out_byte: u8 = 0;
        let mut returned: u32 = 0;

        let ok = unsafe {
            DeviceIoControl(
                self.handle,
                IOCTL_READ_PORT,
                &in_buf as *const _ as *const c_void,
                std::mem::size_of::<ReadPortBuffer>() as u32,
                &mut out_byte as *mut _ as *mut c_void,
                1,
                &mut returned,
                null_mut(),
            )
        };

        if ok != 0 {
            Ok(out_byte)
        } else {
            Err(format!("DeviceIoControl read failed on port 0x{:02X}", port))
        }
    }

    pub fn write_port(&self, port: u8, data: u8) -> Result<(), String> {
        self.transaction(|| self.write_port_raw(port, data))
    }

    pub fn read_port(&self, port: u8) -> Result<u8, String> {
        self.transaction(|| self.read_port_raw(port))
    }
}

impl Drop for PortDriver {
    fn drop(&mut self) {
        unsafe {
            if self.handle != 0 && self.handle != INVALID_HANDLE_VALUE {
                CloseHandle(self.handle);
            }
            if self.named_mutex != 0 && self.named_mutex != INVALID_HANDLE_VALUE {
                CloseHandle(self.named_mutex);
            }
        }
    }
}
