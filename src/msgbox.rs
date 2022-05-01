// Copyright (c) 2022 aiocat
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::ffi::CString;
use std::mem;
use windows::Win32::UI::WindowsAndMessaging::*;

pub unsafe fn error(msg: &str) {
    let title = CString::new("DLL Crab").unwrap();
    let message_c = CString::new(msg).unwrap();

    MessageBoxA(
        None,
        Some(mem::transmute(message_c.as_ptr())),
        Some(mem::transmute(title.as_ptr())),
        MB_ICONERROR | MB_OK,
    );
}

pub unsafe fn info(msg: &str) {
    let title = CString::new("DLL Crab").unwrap();
    let message_c = CString::new(msg).unwrap();

    MessageBoxA(
        None,
        Some(mem::transmute(message_c.as_ptr())),
        Some(mem::transmute(title.as_ptr())),
        MB_ICONINFORMATION | MB_OK,
    );
}
