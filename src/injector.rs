// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT
#![allow(clippy::missing_safety_doc)]

use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::NT_SUCCESS;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{FreeLibrary, GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::{VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{
    CreateRemoteThread, GetExitCodeThread, OpenProcess, OpenThread, QueueUserAPC,
};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use winapi::um::winnt::{
    HANDLE, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, PHANDLE, PROCESS_ALL_ACCESS,
    THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, THREAD_SUSPEND_RESUME,
};

use ntapi::ntpsapi::NtCreateThreadEx;
use ntapi::ntrtl::RtlCreateUserThread;

use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;

// inject dll with CreateRemoteThread method
pub fn inject_create_remote_thread(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = CString::new(dll_path);
    if path_to_dll.is_err() {
        return false;
    }
    let path_to_dll: CString = path_to_dll.unwrap();

    let mut written_bytes = 0;
    let mut thread_id = 0;

    // get process
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false as BOOL, pid) };

    // alloc adress for dll path
    let adress = unsafe {
        VirtualAllocEx(
            process,
            ptr::null_mut(),
            path_to_dll.as_bytes().len() + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    let mut success: bool;
    unsafe {
        // write dll path to process memory
        success = WriteProcessMemory(
            process,
            adress,
            path_to_dll.as_c_str().as_ptr() as *const c_void,
            path_to_dll.as_bytes().len() + 1,
            &mut written_bytes,
        ) != 0;
    }

    // check status
    if !success {
        unsafe {
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }

        return false;
    }

    // get kernel32
    let kernel32_dll = unsafe {
        let kernel32_name = CString::new("kernel32.dll").unwrap();
        GetModuleHandleA(mem::transmute(kernel32_name.as_ptr()))
    };

    // get load library function from kernel32
    let load_library = unsafe {
        let load_library_name = CString::new("LoadLibraryA").unwrap();
        GetProcAddress(kernel32_dll, mem::transmute(load_library_name.as_ptr()))
    };

    // load dll
    let process_thread = unsafe {
        CreateRemoteThread(
            process,
            ptr::null_mut(),
            0,
            Some(mem::transmute(load_library)),
            adress,
            0,
            &mut thread_id,
        )
    };

    // check status
    if process_thread.is_null() {
        unsafe {
            CloseHandle(process_thread);
            FreeLibrary(kernel32_dll);
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }

        return false;
    }

    // wait for thread
    unsafe {
        WaitForSingleObject(process_thread, 0xFFFFFFFF);
    }

    // get thread exit result
    let mut exit_code = 0;

    unsafe {
        if GetExitCodeThread(process_thread, &mut exit_code) == true as BOOL {
            success = true;
        }
    }

    // de-alloc memory, free libraries (memory safety)
    unsafe {
        CloseHandle(process_thread);
        FreeLibrary(kernel32_dll);
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    }

    success
}

// inject dll with RtlCreateUserThread function which is undocumented
pub fn inject_rtl_create_user_thread(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = CString::new(dll_path);
    if path_to_dll.is_err() {
        return false;
    }
    let path_to_dll: CString = path_to_dll.unwrap();

    let mut written_bytes = 0;

    // get process
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false as BOOL, pid) };

    // alloc adress for dll path
    let adress = unsafe {
        VirtualAllocEx(
            process,
            ptr::null_mut(),
            path_to_dll.as_bytes().len() + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    let mut success: bool;
    unsafe {
        // write dll path to process memory
        success = WriteProcessMemory(
            process,
            adress,
            path_to_dll.as_c_str().as_ptr() as *const c_void,
            path_to_dll.as_bytes().len() + 1,
            &mut written_bytes,
        ) != 0;
    }

    // check status
    if !success {
        unsafe {
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }

        return false;
    }

    // get kernel32
    let kernel32_dll = unsafe {
        let kernel32_name = CString::new("kernel32.dll").unwrap();
        GetModuleHandleA(mem::transmute(kernel32_name.as_ptr()))
    };

    // get load library function from kernel32
    let load_library = unsafe {
        let load_library_name = CString::new("LoadLibraryA").unwrap();
        GetProcAddress(kernel32_dll, mem::transmute(load_library_name.as_ptr()))
    };

    // load dll
    let mut process_thread: HANDLE = ptr::null_mut();
    unsafe {
        success = NT_SUCCESS(RtlCreateUserThread(
            process,
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            Some(mem::transmute(load_library)),
            adress,
            &mut process_thread as PHANDLE,
            ptr::null_mut(),
        ));
    }

    // check status
    if !success {
        unsafe {
            CloseHandle(process_thread as HANDLE);
            FreeLibrary(kernel32_dll);
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }
        return false;
    }

    // wait for thread
    unsafe {
        WaitForSingleObject(process_thread as HANDLE, 0xFFFFFFFF);
    }

    // get thread exit result
    let mut exit_code = 0;

    unsafe {
        if GetExitCodeThread(process_thread as HANDLE, &mut exit_code) == true as BOOL {
            success = true;
        }
    }

    // de-alloc memory, free libraries (memory safety)
    unsafe {
        CloseHandle(process_thread as HANDLE);
        FreeLibrary(kernel32_dll);
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    }

    success
}

// inject dll with QueueUserAPC method
pub fn inject_queue_user_apc(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = CString::new(dll_path);
    if path_to_dll.is_err() {
        return false;
    }
    let path_to_dll: CString = path_to_dll.unwrap();
    let mut written_bytes = 0;

    // get process
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false as BOOL, pid) };

    // get tids
    let (tids, mut success) = unsafe { get_tids_by_pid(pid) };
    if !success {
        unsafe {
            CloseHandle(process);
        }
        return false;
    }

    // alloc adress for dll path
    let adress = unsafe {
        VirtualAllocEx(
            process,
            ptr::null_mut(),
            path_to_dll.as_bytes().len() + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    unsafe {
        // write dll path to process memory
        success = WriteProcessMemory(
            process,
            adress,
            path_to_dll.as_c_str().as_ptr() as *const c_void,
            path_to_dll.as_bytes().len() + 1,
            &mut written_bytes,
        ) != 0;
    }

    // check status
    if !success {
        unsafe {
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        };

        return false;
    }

    // get kernel32
    let kernel32_dll = unsafe {
        let kernel32_name = CString::new("kernel32.dll").unwrap();
        GetModuleHandleA(mem::transmute(kernel32_name.as_ptr()))
    };

    // get load library function from kernel32
    let load_library = unsafe {
        let load_library_name = CString::new("LoadLibraryA").unwrap();
        GetProcAddress(kernel32_dll, mem::transmute(load_library_name.as_ptr()))
    };

    // load dll to all threads
    for tid in &tids {
        let process_thread = unsafe {
            OpenThread(
                THREAD_SET_CONTEXT | THREAD_GET_CONTEXT | THREAD_SUSPEND_RESUME,
                false as BOOL,
                *tid,
            )
        };

        // check status
        if process_thread.is_null() {
            unsafe {
                CloseHandle(process_thread);
            };

            success = false;
            break;
        }

        // inject and wait
        unsafe {
            QueueUserAPC(
                Some(mem::transmute(load_library)),
                process_thread,
                adress as usize,
            );
            WaitForSingleObject(process_thread, 0xFFFFFFFF);
        }

        // get thread exit result
        let mut exit_code = 0;

        unsafe {
            if !GetExitCodeThread(process_thread, &mut exit_code) == true as BOOL {
                success = false;
                break;
            }
            CloseHandle(process_thread);
        }
    }

    unsafe {
        FreeLibrary(kernel32_dll);
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    }

    success
}

// inject dll with NtCreateThreadEx function which is undocumented
pub fn inject_nt_create_thread_ex(pid: u32, dll_path: &str) -> bool {
    // c-compatible string for injecting to process memory
    let path_to_dll = CString::new(dll_path);
    if path_to_dll.is_err() {
        return false;
    }
    let path_to_dll: CString = path_to_dll.unwrap();

    let mut written_bytes = 0;

    // get process
    let process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false as BOOL, pid) };

    // alloc adress for dll path
    let adress = unsafe {
        VirtualAllocEx(
            process,
            ptr::null_mut(),
            path_to_dll.as_bytes().len() + 1,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    let mut success: bool;
    unsafe {
        // write dll path to process memory
        success = WriteProcessMemory(
            process,
            adress,
            path_to_dll.as_c_str().as_ptr() as *const c_void,
            path_to_dll.as_bytes().len() + 1,
            &mut written_bytes,
        ) != 0;
    }

    // check status
    if !success {
        unsafe {
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }

        return false;
    }

    // get kernel32
    let kernel32_dll = unsafe {
        let kernel32_name = CString::new("kernel32.dll").unwrap();
        GetModuleHandleA(mem::transmute(kernel32_name.as_ptr()))
    };

    // get load library function from kernel32
    let load_library = unsafe {
        let load_library_name = CString::new("LoadLibraryA").unwrap();
        GetProcAddress(kernel32_dll, mem::transmute(load_library_name.as_ptr()))
    };

    // load dll
    let mut process_thread: HANDLE = ptr::null_mut();
    unsafe {
        success = NT_SUCCESS(NtCreateThreadEx(
            &mut process_thread,
            0x1FFFFF,
            ptr::null_mut(),
            process,
            mem::transmute(load_library),
            adress,
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        ));
    }

    // check status
    if !success {
        unsafe {
            CloseHandle(process_thread as HANDLE);
            FreeLibrary(kernel32_dll);
            VirtualFreeEx(process, adress, 0, MEM_RELEASE);
            CloseHandle(process);
        }
        return false;
    }

    // wait for thread
    unsafe {
        WaitForSingleObject(process_thread as HANDLE, 0xFFFFFFFF);
    }

    // get thread exit result
    let mut exit_code = 0;

    unsafe {
        if GetExitCodeThread(process_thread as HANDLE, &mut exit_code) == true as BOOL {
            success = true;
        }
    }

    // de-alloc memory, free libraries (memory safety)
    unsafe {
        CloseHandle(process_thread as HANDLE);
        FreeLibrary(kernel32_dll);
        VirtualFreeEx(process, adress, 0, MEM_RELEASE);
        CloseHandle(process);
    }

    success
}

// list tids from pid
unsafe fn get_tids_by_pid(pid: u32) -> (Vec<DWORD>, bool) {
    let mut tids: Vec<DWORD> = Vec::new();

    let mut entry: THREADENTRY32 = THREADENTRY32 {
        dwSize: 0_u32,
        cntUsage: 0_u32,
        th32ThreadID: 0_u32,
        th32OwnerProcessID: 0_u32,
        tpBasePri: 0,
        tpDeltaPri: 0,
        dwFlags: 0_u32,
    };
    entry.dwSize = mem::size_of_val(&entry) as u32;

    let handle_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);

    if Thread32First(handle_snapshot, &mut entry) != 0 {
        loop {
            if pid == entry.th32OwnerProcessID {
                tids.push(entry.th32ThreadID);
            }

            if Thread32Next(handle_snapshot, &mut entry) == 0 {
                break;
            }
        }
    }

    if tids.is_empty() {
        return (tids, false);
    }

    CloseHandle(handle_snapshot);
    (tids, true)
}
