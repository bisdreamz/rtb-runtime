//! Build script for OpenRTB Rust bindings
//!
//! This script handles the compilation of OpenRTB protobuf definitions into Rust code.
//! It works around prost's lack of support for Protobuf Editions by patching the proto
//! files at build time.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

/// Patches OpenRTB proto file to be compatible with prost
///
/// OpenRTB uses Protobuf Editions (edition = "2023") which prost doesn't support yet.
/// See: https://github.com/tokio-rs/prost/issues/1031
///
/// This function:
/// - Converts `edition = "2023"` to `syntax = "proto3"`
/// - Removes extension ranges (unused in OpenRTB, proto2-only syntax)
/// - Strips default values (proto2-only, not allowed in proto3)
/// - Adds missing import for google.protobuf.Value types
fn patch_proto(src: &str) -> String {
    let has_import = src.contains("google/protobuf/struct.proto");

    let mut out = String::new();
    let mut inserted_syntax = false;
    let mut inserted_import = false;
    let mut in_header = true;

    for line in src.lines() {
        let trimmed = line.trim();

        // Strip edition syntax (incompatible with prost)
        if line.starts_with("edition =") {
            continue;
        }

        // Strip option features.* (editions-specific)
        if trimmed.starts_with("option features.") {
            continue;
        }

        // Strip extension ranges (unused placeholders, proto2-only)
        // OpenRTB defines these but never uses them - safe to remove
        if trimmed.starts_with("extensions ") && trimmed.contains(" to ") {
            continue;
        }

        // Strip explicit default values (proto2-only, not allowed in proto3)
        // Example: `optional int32 field = 1 [default = 0];`
        // This is safe because deprecated fields are on separate lines in OpenRTB proto
        let line_to_write = if trimmed.contains("[default") {
            let before_bracket = line.split('[').next().unwrap_or(line);
            let after_bracket = line.split(']').skip(1).collect::<Vec<_>>().join("]");
            let trimmed_result = format!("{}{}", before_bracket, after_bracket)
                .trim_end()
                .to_string();
            // Add back semicolon if it was there
            if line.trim_end().ends_with(';') && !trimmed_result.ends_with(';') {
                format!("{};", trimmed_result)
            } else {
                trimmed_result
            }
        } else {
            line.to_string()
        };

        // Insert syntax = "proto3" at the very top (after comments)
        if !inserted_syntax && !trimmed.is_empty() && !trimmed.starts_with("//") {
            out.push_str("syntax = \"proto3\";\n");
            inserted_syntax = true;
        }

        // Check if we're still in the header section
        if in_header && !trimmed.is_empty() && !trimmed.starts_with("//") {
            if !(line.starts_with("syntax =")
                || line.starts_with("package ")
                || line.starts_with("import ")
                || line.starts_with("option "))
            {
                // We've left the header - insert import before first message/enum
                if !has_import && !inserted_import {
                    out.push_str("import \"google/protobuf/struct.proto\";\n\n");
                    inserted_import = true;
                }
                in_header = false;
            }
        }

        out.push_str(&line_to_write);
        out.push('\n');
    }

    // If we never found a place to insert the import, add it at the end
    if !has_import && !inserted_import {
        out.push_str("\nimport \"google/protobuf/struct.proto\";\n");
    }

    out
}

/// Copies and patches the OpenRTB proto file for prost compilation
fn copy_and_patch_proto(
    src_path: &Path,
    dst_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let src = fs::read_to_string(src_path)
        .map_err(|e| format!("Failed to read OpenRTB proto at {:?}: {}", src_path, e))?;

    let patched = patch_proto(&src);

    let dst_path = dst_dir.join("openrtb_patched.proto");
    fs::write(&dst_path, patched).map_err(|e| format!("Failed to write patched proto: {}", e))?;

    Ok(dst_path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use vendored protoc to avoid requiring system installation
    let protoc = protoc_bin_vendored::protoc_bin_path()?
        .to_string_lossy()
        .into_owned();
    let inc = protoc_bin_vendored::include_path()?;

    // Set PROTOC environment variable for prost
    // SAFETY: This is safe because we're single-threaded in build.rs
    // and setting it before any protoc invocation
    unsafe {
        env::set_var("PROTOC", protoc);
    }

    // OpenRTB proto location (git submodule)
    let root_inc = Path::new("openrtb2.x/src/main");
    let openrtb_proto = root_inc.join("com/iabtechlab/openrtb/v2/openrtb.proto");

    // Verify the proto file exists
    if !openrtb_proto.exists() {
        return Err(format!(
            "OpenRTB proto not found at {:?}. Did you forget to run 'git submodule update --init'?",
            openrtb_proto
        )
        .into());
    }

    // Write a patched copy into OUT_DIR and compile that
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let patched = copy_and_patch_proto(&openrtb_proto, &out_dir)?;

    // Rebuild if the source proto changes
    println!("cargo:rerun-if-changed={}", openrtb_proto.display());

    let descriptor_path = out_dir.join("descriptor.bin");

    tonic_prost_build::configure()
        // Use extern path for well-known types
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::pbjson_types")
        // Add builder pattern support for easier construction (structs only)
        .message_attribute(".", "#[derive(derive_builder::Builder)]")
        .message_attribute(".", "#[builder(setter(into, strip_option), default)]")
        // Emit file descriptor for pbjson
        .file_descriptor_set_path(&descriptor_path)
        // Add include path for well-known types
        .protoc_arg(format!("-I{}", inc.display()))
        // Compile the proto
        .compile_protos(
            &[patched.to_string_lossy().to_string()],
            &[
                out_dir.to_string_lossy().to_string(),
                root_inc.to_string_lossy().to_string(),
            ],
        )?;

    // Generate serde implementations with pbjson
    let descriptor_set = std::fs::read(&descriptor_path)?;

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .preserve_proto_field_names() // Keep original field names (not camelCase)
        .emit_fields() // Allow field number access
        .build(&[".com.iabtechlab.openrtb.v2"])?;

    Ok(())
}
