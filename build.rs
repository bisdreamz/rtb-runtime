use std::{env, fs, path::{Path, PathBuf}};

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
        if trimmed.starts_with("extensions ") && trimmed.contains(" to ") {
            continue;
        }

        // Strip explicit default values (proto2-only, not allowed in proto3)
        // Simple approach: if a line has [default = X], remove the whole annotation
        // This is safe because deprecated fields are on separate lines in OpenRTB proto
        let line_to_write = if trimmed.contains("[default") {
            let before_bracket = line.split('[').next().unwrap_or(line);
            let after_bracket = line.split(']').skip(1).collect::<Vec<_>>().join("]");
            let trimmed_result = format!("{}{}", before_bracket, after_bracket).trim_end().to_string();
            // Add back semicolon if it was there
            if line.trim_end().ends_with(';') && !trimmed_result.ends_with(';') {
                format!("{};", trimmed_result)
            } else {
                trimmed_result
            }
        } else {
            line.to_string()
        };

        // Insert syntax = "proto3" at the very top
        if !inserted_syntax && !trimmed.is_empty() && !trimmed.starts_with("//") {
            out.push_str("syntax = \"proto3\";\n");
            inserted_syntax = true;
        }

        // Check if we're still in the header section
        if in_header && !trimmed.is_empty() && !trimmed.starts_with("//") {
            if !(line.starts_with("syntax =") || line.starts_with("package ")
                || line.starts_with("import ") || line.starts_with("option ")) {
                // We've left the header - insert import before this line
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

    if !has_import && !inserted_import {
        out.push_str("\nimport \"google/protobuf/struct.proto\";\n");
    }

    out
}

fn copy_and_patch_proto(src_path: &Path, dst_dir: &Path) -> PathBuf {
    let src = fs::read_to_string(src_path)
        .expect("read openrtb.proto");
    let patched = patch_proto(&src);

    let dst_path = dst_dir.join("openrtb_patched.proto");
    fs::write(&dst_path, patched).expect("write patched proto");
    dst_path
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Vendored protoc and its include dir (contains well-known types)
    let protoc = protoc_bin_vendored::protoc_bin_path()?
        .to_string_lossy()
        .into_owned();
    let inc = protoc_bin_vendored::include_path()?;

    unsafe {
        env::set_var("PROTOC", protoc);
    }

    // Paths in your repo
    let root_inc = Path::new("openrtb2.x/src/main");
    let openrtb_proto = root_inc.join("com/iabtechlab/openrtb/v2/openrtb.proto");

    // Write a patched copy into OUT_DIR and compile that
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let patched = copy_and_patch_proto(&openrtb_proto, &out_dir);

    // Rebuild if the source proto changes
    println!("cargo:rerun-if-changed={}", openrtb_proto.display());

    tonic_prost_build::configure()
        // don't re-generate WKTs; point to prost-types
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        // Enable serde support for all generated types
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]")
        // add vendored WKT include
        .protoc_arg(format!("-I{}", inc.display()))
        // also include the repo's proto root so other imports still resolve
        .compile_protos(
            &[patched.to_string_lossy().to_string()],
            &[out_dir.to_string_lossy().to_string(), root_inc.to_string_lossy().to_string()],
        )?;

    Ok(())
}
