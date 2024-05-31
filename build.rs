// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;

fn main() {
    let _dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    #[cfg(target_arch = "x86")]
    {
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&dir).join("lib/x86").display()
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&dir).join("lib/x64").display()
        );
    }
}
