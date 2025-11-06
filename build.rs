//! Build script for OpenRTB Rust bindings
//!
//! This script handles the compilation of OpenRTB protobuf definitions into Rust code.
//! It works around prost's lack of support for Protobuf Editions by patching the proto
//! files at build time.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use heck::{ToSnakeCase, ToUpperCamelCase};
use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorSet, field_descriptor_proto::Type as FieldType};

/// Patches OpenRTB proto file to be compatible with prost, until editions support exists
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

    prost_build::Config::new()
        // Use extern path for well-known types
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        // Restore builder derives on all generated message structs
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
        .ignore_unknown_fields()
        .build(&[".com.iabtechlab.openrtb.v2"])?;

    let bool_fields = collect_bool_field_names(&descriptor_set)?;
    let ext_fields = collect_ext_field_info(&descriptor_set)?;
    let serde_path = out_dir.join("com.iabtechlab.openrtb.v2.serde.rs");
    patch_pbjson_bool_handling(&serde_path, &bool_fields)?;
    patch_inline_hints(&serde_path)?;

    let proto_path = out_dir.join("com.iabtechlab.openrtb.v2.rs");
    patch_ext_wrapper(&proto_path, &ext_fields)?;

    Ok(())
}

fn collect_bool_field_names(
    descriptor_bytes: &[u8],
) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn std::error::Error>> {
    let descriptor_set = FileDescriptorSet::decode(descriptor_bytes)?;
    let mut fields: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for file in descriptor_set.file {
        if file.package.as_deref() != Some("com.iabtechlab.openrtb.v2") {
            continue;
        }
        for message in file.message_type {
            let mut path = Vec::new();
            collect_from_message(&message, &mut path, &mut fields);
        }
    }

    Ok(fields)
}

fn collect_from_message(
    message: &DescriptorProto,
    path: &mut Vec<String>,
    fields: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let name = match &message.name {
        Some(name) => name.clone(),
        None => return,
    };

    path.push(name);

    for field in &message.field {
        if field.r#type == Some(FieldType::Bool as i32) {
            if let Some(field_name) = &field.name {
                let type_path = rust_type_path(path);
                fields
                    .entry(type_path)
                    .or_default()
                    .insert(field_name.clone());
            }
        }
    }

    for nested in &message.nested_type {
        if nested
            .options
            .as_ref()
            .and_then(|opt| opt.map_entry)
            .unwrap_or(false)
        {
            continue;
        }
        collect_from_message(nested, path, fields);
    }

    path.pop();
}

fn rust_type_path(path: &[String]) -> String {
    if path.is_empty() {
        return String::new();
    }

    let mut modules = Vec::new();
    for segment in path.iter().take(path.len() - 1) {
        modules.push(segment.to_snake_case());
    }

    let type_name = path.last().unwrap().clone();

    if modules.is_empty() {
        type_name
    } else {
        format!("{}::{}", modules.join("::"), type_name)
    }
}

fn patch_pbjson_bool_handling(
    serde_path: &Path,
    bool_fields: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = fs::read_to_string(serde_path)
        .map_err(|e| format!("failed to read generated serde file: {e}"))?;

    let mut lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let mut serialize_hits = 0usize;
    let mut deserialize_hits = 0usize;

    let mut i = 0usize;
    while i < lines.len() {
        if let Some(type_name) = extract_impl_type(&lines[i], "impl serde::Serialize for ") {
            let fields = bool_fields
                .get(&type_name)
                .map(|set| set.iter().cloned().collect::<Vec<_>>());
            let mut depth = brace_delta(&lines[i]);
            let mut j = i + 1;
            while depth > 0 && j < lines.len() {
                if let Some(fields) = &fields {
                    for field in fields {
                        let needle =
                            format!("struct_ser.serialize_field(\"{field}\", &self.{field})?;");
                        if lines[j].contains(&needle) {
                            let replacement = format!(
                                "struct_ser.serialize_field(\"{field}\", &crate::compat::bool_as_int::Ser(&self.{field}))?;"
                            );
                            lines[j] = lines[j].replace(&needle, &replacement);
                            serialize_hits += 1;
                        }
                    }
                }
                depth += brace_delta(&lines[j]);
                j += 1;
            }
            i = j;
            continue;
        }

        if let Some(type_name) =
            extract_impl_type(&lines[i], "impl<'de> serde::Deserialize<'de> for ")
        {
            let fields = bool_fields
                .get(&type_name)
                .map(|set| set.iter().cloned().collect::<Vec<_>>());
            let mut depth = brace_delta(&lines[i]);
            let mut j = i + 1;
            while depth > 0 && j < lines.len() {
                if let Some(fields) = &fields {
                    for field in fields {
                        let pattern = format!("{field}__ = Some(map_.next_value()?);");
                        if lines[j].contains(&pattern) {
                            let replacement = format!(
                                "{field}__ = Some(map_.next_value::<crate::compat::bool_as_int::De>()?.0);"
                            );
                            lines[j] = lines[j].replace(&pattern, &replacement);
                            deserialize_hits += 1;
                        } else {
                            let direct_pattern = format!("{field}__ = map_.next_value()?;");
                            if lines[j].contains(&direct_pattern) {
                                let replacement = format!(
                                    "{field}__ = map_.next_value::<crate::compat::bool_as_int::De>()?.0;"
                                );
                                lines[j] = lines[j].replace(&direct_pattern, &replacement);
                                deserialize_hits += 1;
                            }
                        }
                    }
                }
                depth += brace_delta(&lines[j]);
                j += 1;
            }
            i = j;
            continue;
        }

        i += 1;
    }

    if serialize_hits == 0 || deserialize_hits == 0 {
        return Err(format!(
            "failed to patch pbjson output for bool fields (serialize_hits={serialize_hits}, deserialize_hits={deserialize_hits})"
        )
        .into());
    }

    let mut output = lines.join("\n");
    output.push('\n');
    fs::write(serde_path, output)
        .map_err(|e| format!("failed to write patched serde file: {e}"))?;

    Ok(())
}

#[derive(Debug, Clone)]
struct ExtFieldInfo {
    struct_path: String,
    ext_type_path: String,
    rust_struct_name: String,
}

fn collect_ext_field_info(
    descriptor_bytes: &[u8],
) -> Result<Vec<ExtFieldInfo>, Box<dyn std::error::Error>> {
    let descriptor_set = FileDescriptorSet::decode(descriptor_bytes)?;
    let mut fields = Vec::new();

    for file in descriptor_set.file {
        if file.package.as_deref() != Some("com.iabtechlab.openrtb.v2") {
            continue;
        }
        for message in file.message_type {
            let mut path = Vec::new();
            collect_ext_from_message(&message, &mut path, &mut fields);
        }
    }

    Ok(fields)
}

fn collect_ext_from_message(
    message: &DescriptorProto,
    path: &mut Vec<String>,
    fields: &mut Vec<ExtFieldInfo>,
) {
    let name = match &message.name {
        Some(name) => name.clone(),
        None => return,
    };

    path.push(name);

    for field in &message.field {
        if field.name.as_deref() == Some("ext") {
            if let Some(type_name) = &field.type_name {
                if type_name.ends_with(".Ext") {
                    let struct_path = rust_type_path(path);
                    let proto_name = path.last().unwrap();
                    let ext_module = proto_name.to_snake_case();
                    let ext_type_path = format!("{}::Ext", ext_module);
                    let rust_struct_name = proto_name.to_upper_camel_case();
                    fields.push(ExtFieldInfo {
                        struct_path,
                        ext_type_path,
                        rust_struct_name,
                    });
                }
            }
        }
    }

    for nested in &message.nested_type {
        if nested
            .options
            .as_ref()
            .and_then(|opt| opt.map_entry)
            .unwrap_or(false)
        {
            continue;
        }
        collect_ext_from_message(nested, path, fields);
    }

    path.pop();
}

fn extract_impl_type(line: &str, prefix: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix(prefix) {
        return Some(rest.split('{').next()?.trim().to_owned());
    }
    None
}

fn brace_delta(line: &str) -> i32 {
    line.chars().fold(0, |acc, ch| match ch {
        '{' => acc + 1,
        '}' => acc - 1,
        _ => acc,
    })
}

fn patch_inline_hints(serde_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let code = fs::read_to_string(serde_path)?;
    let mut lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();

    let hot_types = [
        "BidRequest",
        "bid_request::Imp",
        "bid_request::Device",
        "bid_request::User",
    ];

    let mut in_hot_impl = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].clone();

        if line
            .trim()
            .starts_with("impl<'de> serde::Deserialize<'de> for ")
        {
            in_hot_impl = hot_types.iter().any(|t| line.contains(t));
        }

        if in_hot_impl && line.trim().starts_with("fn deserialize<D>(") {
            if i > 0 && !lines[i - 1].trim().starts_with("#[inline]") {
                lines.insert(i, "    #[inline]".to_string());
                i += 1;
            }
            in_hot_impl = false;
        }

        if line.trim().starts_with("fn visit_map<") {
            if i > 0 && !lines[i - 1].trim().starts_with("#[inline]") {
                lines.insert(i, "            #[inline]".to_string());
                i += 1;
            }
        }

        i += 1;
    }

    let mut output = lines.join("\n");
    output.push('\n');
    fs::write(serde_path, output)?;

    Ok(())
}

/// Patches generated proto code to wrap extension fields with ExtWithCustom.
///
/// This function uses descriptor-driven metadata to reliably identify every
/// OpenRTB `ext` field and wraps it with `ExtWithCustom<T>` so we preserve both
/// proto-defined fields and dynamic JSON data.
fn patch_ext_wrapper(
    proto_path: &Path,
    ext_fields: &[ExtFieldInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    if ext_fields.is_empty() {
        return Ok(());
    }

    let code = fs::read_to_string(proto_path)
        .map_err(|e| format!("failed to read generated proto file for ext patching: {e}"))?;

    let mut lines: Vec<String> = code.lines().map(|s| s.to_string()).collect();
    let mut replacements = 0usize;
    let mut copy_removed: BTreeSet<String> = BTreeSet::new();

    for field in ext_fields {
        let search = format!("::core::option::Option<{}>", field.ext_type_path);
        let replacement = format!(
            "::core::option::Option<crate::extensions::ExtWithCustom<{}>>",
            field.ext_type_path
        );

        let mut found_idx = None;
        for (idx, line) in lines.iter_mut().enumerate() {
            if line.contains(&search) {
                if !line.contains("ExtWithCustom") {
                    *line = line.replace(&search, &replacement);
                    replacements += 1;
                }
                found_idx = Some(idx);
                break;
            }
        }

        let field_line_idx = match found_idx {
            Some(idx) => idx,
            None => {
                return Err(format!(
                    "Failed to locate ext field declaration for {}",
                    field.ext_type_path
                )
                .into());
            }
        };

        if copy_removed.insert(field.struct_path.clone()) {
            remove_copy_from_struct(&mut lines, field_line_idx, &field.rust_struct_name)?;
        }
    }

    let remaining = lines
        .iter()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("pub ext: ::core::option::Option<")
                && trimmed.contains("::Ext>")
                && !trimmed.contains("ExtWithCustom")
        })
        .count();

    if remaining > 0 {
        return Err(format!(
            "Verification failed: {} unwrapped ext fields remain after patching",
            remaining
        )
        .into());
    }

    let mut output = lines.join("\n");
    output.push('\n');
    fs::write(proto_path, output)
        .map_err(|e| format!("failed to write ext-patched proto file: {e}"))?;

    println!(
        "cargo:warning=Patched {} ext fields to use ExtWithCustom",
        replacements
    );

    Ok(())
}

fn remove_copy_from_struct(
    lines: &mut [String],
    start_idx: usize,
    struct_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if struct_name.is_empty() {
        return Ok(());
    }

    let mut struct_idx = None;
    for i in (0..=start_idx).rev() {
        if lines[i].contains(&format!("pub struct {}", struct_name)) {
            struct_idx = Some(i);
            break;
        }
    }

    if let Some(struct_idx) = struct_idx {
        for i in (0..struct_idx).rev() {
            let trimmed = lines[i].trim();
            if trimmed.starts_with("#[derive(") {
                if lines[i].contains("Copy") {
                    let updated = lines[i]
                        .replace(", Copy", "")
                        .replace("Copy, ", "")
                        .replace("Copy", "");
                    lines[i] = updated;
                }
                break;
            }

            if trimmed.starts_with("pub struct") || trimmed.starts_with("pub enum") {
                break;
            }
        }
    }

    Ok(())
}
