// Copyright (c) 2022 aiocat
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use winres::{WindowsResource, VersionInfo};

fn main() {
    let mut res = WindowsResource::new();
    res.set_icon("./assets/dll-crab.ico")
    .set("InternalName", "DLL Crab")
    .set_version_info(VersionInfo::PRODUCTVERSION, 0x0001000000000000);

    res.compile().unwrap();
}