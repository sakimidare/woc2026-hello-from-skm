// SPDX-License-Identifier: GPL-2.0

//! SKM is a simple linux kernel module written in rust.

use kernel::prelude::*;

module! {
    type: SASTKernelModule,
    name: "woc2026_hello_from_skm",
    authors: ["fermata"],
    description: "SKM is a simple linux kernel module written in rust",
    license: "GPL",
}

struct SASTKernelModule;

impl kernel::Module for SASTKernelModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Welcome to SAST!\n");
        pr_info!("Am I built-in? {}\n", !cfg!(MODULE));

        // 使用 panic! 明确触发 panic
        panic!("Intentional panic for testing!");

        #[allow(unreachable_code)]
        Ok(SASTKernelModule)
    }
}

impl Drop for SASTKernelModule {
    fn drop(&mut self) {
        pr_info!("bye bye\n");
    }
}
