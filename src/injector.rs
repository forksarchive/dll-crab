// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use winapi::shared::minwindef::BOOL;
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{FreeLibrary, GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::{VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThread, GetExitCodeThread, OpenProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, PROCESS_ALL_ACCESS};

use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;

pub fn inject_dll(pid: u32, dll_path: &str) -> bool {
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

    unsafe {
        // write dll path to process memory
        WriteProcessMemory(
            process,
            adress,
            path_to_dll.as_c_str().as_ptr() as *const c_void,
            path_to_dll.as_bytes().len() + 1,
            &mut written_bytes,
        );
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

    // wait for thread
    unsafe {
        WaitForSingleObject(process_thread, 0xFFFFFFFF);
    }

    // get thread exit result
    let mut exit_code = 0;
    let mut success: bool = false;

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
