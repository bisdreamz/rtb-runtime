# OpenRTB Compatibility Layer

This directory contains internal utilities that bridge the gap between OpenRTB's protobuf definitions and its JSON specification. These components run at build time and runtime to ensure generated Rust code conforms to OpenRTB's JSON requirements.

## Overview

The OpenRTB specification has several characteristics that don't map cleanly to standard protobuf semantics:

1. **Boolean fields serialize as integers**: Many fields defined as `bool` in the proto must serialize as `0` or `1` in JSON, not `true` or `false`.

2. **Extension fields allow arbitrary JSON**: The proto uses `extensions 500 to max;` declarations to permit custom fields, but prost (our protobuf library) doesn't support this feature yet.

3. **The upstream schema targets protobuf editions**: OpenRTB 2.x is published as `edition = "2023"`, a format prost does not yet implement.

This compatibility layer addresses all three by rewriting the generated code at build
time and providing small runtime helpers.

## Protobuf Adjustments

Before code generation we normalise the upstream proto so prost can ingest it:

- replace `edition = "2023"` with `syntax = "proto3"`
- strip edition-only options (e.g. `option features.*`)
- remove empty extension ranges (`extensions 500 to max;`)
- drop explicit default annotations (`[default = …]`), which proto3 forbids

The patching happens inside `build.rs` and is idempotent. When prost eventually
supports Editions this step can be removed.

Note that standard protobuf encoding still discards unknown fields. Custom
extension data is preserved for JSON via `ExtWithCustom`, but gRPC (binary)
traffic only carries the schema-defined portion of each `ext` object.

## Architecture

### Build Time (build.rs)

The build script generates Rust code from OpenRTB protobuf definitions, then patches the generated serde implementations to use our compatibility helpers:

```
Proto Definition → Prost Code Gen → Pbjson Serde Gen → Patch Serde Impls → Final Code
```

Two patching passes occur:

1. **Boolean field patching**: Wraps specific bool fields with `bool_as_int` helpers
2. **Extension field patching**: Replaces `ext` field types with `ExtWithCustom<T>` wrappers

### Runtime (this module)

The patched code references types from this module during serialization and deserialization:

- `bool_as_int::Ser` / `bool_as_int::De` - Boolean conversion helpers
- `extensions::ExtWithCustom<T>` - Extension field wrapper
- `extensions::DynamicExt` - Dynamic field storage

## File Descriptions

### bool_as_int.rs

**Purpose**: Serde wrappers that convert between Rust `bool` and JSON integers (`0`/`1`).

**Why it exists**: OpenRTB specifies that many boolean fields must serialize as integers in JSON. The proto definitions use `bool` types for type safety and ergonomics in code, but the JSON wire format requires integers for compatibility with existing implementations.

**How it works**: At build time we walk the protobuf descriptor, record every
`bool` field under `com.iabtechlab.openrtb.v2`, and wrap those fields with the
helpers in the generated serde implementations. For example:

```rust
// Before patching (generated):
struct_ser.serialize_field("skip", &self.skip)?;

// After patching:
struct_ser.serialize_field("skip", &crate::compat::bool_as_int::Ser(&self.skip))?;
```

The `Ser` wrapper intercepts serialization and outputs `0` or `1`. The `De` wrapper accepts multiple input formats (integers, booleans, strings) for maximum compatibility with varied OpenRTB implementations.

**Fields affected**: `test`, `allimps`, `secure`, `instl`, `skip`, `topframe`,
`dnt`, `lmt`, `coppa`, `gdpr`, and many others. The exact list comes from the
descriptor, so future schema changes are handled automatically.

**Usage**: End users never interact with this module directly. It operates transparently through serde.

### extensions.rs

**Purpose**: Provides type-safe access to both proto-defined and custom extension fields.

**Why it exists**: OpenRTB uses protobuf extensions to allow custom fields beyond the core specification. However, prost doesn't support the `extensions` keyword yet (see [prost#674](https://github.com/tokio-rs/prost/issues/674)). Without special handling, custom fields would be silently dropped or cause deserialization failures.

**How it works**: At build time, all `ext` field declarations are modified from:

```rust
pub ext: Option<imp::Ext>
```

To:

```rust
pub ext: Option<crate::extensions::ExtWithCustom<imp::Ext>>
```

The `ExtWithCustom<T>` type combines:
- the original proto type `T` containing standard fields (gpid, skadn, etc.)
- a `DynamicExt` map containing unknown/custom fields captured during JSON
  deserialization

The `Deref` implementation makes the wrapper feel transparent for schema
fields, while the JSON serializer emits both proto and custom values. When the
same type is encoded as protobuf only the proto portion is transmitted (custom
entries have no corresponding field numbers), mirroring the behaviour of the
official schema.

**API Design**:

```rust
// Proto fields (via Deref):
let gpid = request.imp[0].ext.gpid;
let skadn = request.imp[0].ext.skadn;

// Custom fields (explicit method):
let channel = request.imp[0].ext.custom().get_i64("channel");
```

This design provides:
- IDE autocomplete for proto fields
- Type safety for proto fields
- Flexible access to custom fields
- Preservation of all fields during round-trip serialization

**DynamicExt API**: The `DynamicExt` type provides convenience methods for common operations:

- **Primitives**: `get_bool()`, `get_i64()`, `get_u64()`, `get_f64()`, `get_str()`
- **Defaults**: `get_i64_or()`, `get_bool_or()`, etc.
- **Collections**: `get_array()`, `get_object()`, `get_nested()`
- **Typed access**: `get_as::<T>()`, `get_array_as::<T>()`, `as_typed::<T>()`

**Protobuf Support**: `ExtWithCustom<T>` implements `prost::Message` to enable full gRPC/tonic compatibility. The implementation delegates all protobuf operations to the inner proto type:

```rust
impl<T: Message + Default> Message for ExtWithCustom<T> {
    fn encode_raw(&self, buf: &mut impl BufMut) {
        // Only encode proto fields; custom fields ignored
        self.proto.encode_raw(buf);
    }
    // ... other methods delegate similarly
}
```

This allows the same types to work with both transports:
- **JSON**: Custom fields captured via serde `#[serde(flatten)]`
- **Protobuf**: Custom fields ignored (don't exist in proto schema)

The unified API means business logic works regardless of transport:

```rust
// Works with both JSON and protobuf sources
fn extract_channel(request: &BidRequest) -> Option<i64> {
    request.imp.first()?.ext.as_ref()?.custom().get_i64("channel")
}

// From JSON: returns Some(42) if field exists
// From protobuf: returns None (custom fields not in schema)
```

**Additional Traits**: To support protobuf-generated structs that derive `Copy`, `Eq`, or `Hash`, `ExtWithCustom<T>` provides implementations:
- `Hash`: Hashes both proto and custom fields deterministically
- `Eq`: Delegates to inner `PartialEq` implementation
- `Copy`: Not implemented (HashMap can't be Copy); build script removes `Copy` from affected structs

**Usage**: Users interact with this through the public `extensions` module exported at the crate root.

## Integration Points

### Generated Code Dependencies

The patched serde implementations reference these modules:

```rust
// In generated deserialization code:
field__ = Some(map_.next_value::<crate::compat::bool_as_int::De>()?.0);

// In generated struct definitions:
pub ext: Option<crate::extensions::ExtWithCustom<video::Ext>>
```

### Public API Surface

Only the `extensions` module is exposed publicly:

```rust
// In src/lib.rs:
pub mod extensions {
    pub use crate::compat::extensions::*;
}
```

The `bool_as_int` module remains internal as users never interact with it directly.

## Future Considerations

### Protobuf Editions Support

When prost adds support for Protobuf Editions (the format OpenRTB uses), the proto patching in build.rs may need adjustment. The extension handling approach should remain valid.

### Native Extension Support

If prost implements proper extension support (issue #674), we could potentially simplify or remove the `ExtWithCustom` wrapper. However, the current approach provides ergonomic benefits even beyond working around prost limitations.

### Performance

The current implementation has negligible overhead:
- Boolean conversion happens at the serde level (no post-processing)
- Extension wrappers use `#[serde(flatten)]` for efficient parsing
- `Deref` to proto fields is zero-cost

Benchmarks show the full serialization/deserialization pipeline performs at over 150K requests per second on typical hardware.

## Testing

Both modules include comprehensive unit tests:

- `bool_as_int`: Tests integer/bool conversion in both directions
- `extensions`: Tests primitive access, nested objects, typed deserialization, and round-trip serialization

Integration tests in `tests/` verify the full build → parse → access workflow with real OpenRTB JSON samples.

## Maintenance

When updating the OpenRTB proto dependency:

1. Run `cargo build` and check for patching warnings
2. Review build.rs output for the count of patched boolean fields
3. Verify integration tests pass with new proto version
4. Check that the extension field structure hasn't changed

The build.rs script includes diagnostics to detect proto structure changes that might require code updates.
