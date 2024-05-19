// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{ffi::OsStr, path::Path};

use shaderc;

fn main() {
    let compiler = shaderc::Compiler::new().unwrap();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let spv_dir = Path::new(&out_dir).join("shaders/");
    std::fs::create_dir_all(&spv_dir).expect(&format!("{spv_dir:?}"));
    for path in std::fs::read_dir("shaders").unwrap() {
        let path = path.unwrap().path();
        let kind = if path.extension() == Some(OsStr::new("vert")) {
            shaderc::ShaderKind::Vertex
        } else if path.extension() == Some(OsStr::new("frag")) {
            shaderc::ShaderKind::Fragment
        } else {
            panic!(
                "Shader file extension must be .vert or .frag, got {:?}",
                path
            )
        };
        let src_path = path.to_str().unwrap();
        let source = std::fs::read_to_string(&path).expect(src_path);
        let binary_result = compiler
            .compile_into_spirv(&source, kind, src_path, "main", None)
            .expect(src_path);
        let dest_path =
            Path::new(&spv_dir).join(path.file_name().expect(src_path));
        std::fs::write(dest_path, binary_result.as_binary_u8())
            .expect(src_path);
        println!("cargo::rerun-if-changed={src_path}");
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-rpath");
        println!("cargo:rustc-link-arg=@executable_path/../Frameworks");
    }
}
