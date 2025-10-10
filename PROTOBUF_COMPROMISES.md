# OpenRTB Protobuf Build Compromises

## Why This Document Exists

The OpenRTB 2.x protobuf specification uses **Protobuf Editions** (`edition = "2023"`), which provides modern features like extension ranges and explicit defaults. However, **prost** (the Rust protobuf library used by tonic) does not yet support Editions syntax as of October 2025.

**Tracking issue:** https://github.com/tokio-rs/prost/issues/1031

## Our Approach

We patch the OpenRTB proto file at build time to convert it from Editions to `syntax = "proto3"`, which prost fully supports. This is a **temporary workaround** until prost gains editions support.

## What We Strip

### 1. Edition Syntax
```protobuf
edition = "2023";  // STRIPPED
```
**Replaced with:** `syntax = "proto3";`

### 2. Extension Ranges
```protobuf
message BidRequest {
  extensions 500 to max;  // STRIPPED
}
```
**Why safe:** The OpenRTB proto defines these ranges but **never actually uses them**. We verified zero `extend` statements exist in the file.

### 3. Explicit Default Values
```protobuf
optional int32 at = 1 [default = 2];  // [default = 2] STRIPPED
```
**Why safe:** Proto3 doesn't support explicit defaults. We handle default logic in application code.

### 4. Feature Options
```protobuf
option features.field_presence = EXPLICIT;  // STRIPPED
```
**Why safe:** Editions-specific configuration not needed for our use case.

## What We Preserve

### ✅ All Strongly-Typed Messages
- `BidRequest`, `BidResponse`, `Impression`, etc.
- `SupplyChain` and `SupplyChainNode`
- `EID` (Extended Identifiers)
- All OpenRTB 2.6 spec messages

### ✅ All Regular Extension Fields
```protobuf
message Source {
  message Ext {
    SupplyChain schain = 1;   // ✅ PRESERVED
    string omidpn = 2;         // ✅ PRESERVED
    string omidpv = 3;         // ✅ PRESERVED
  }
}
```

These are **regular protobuf fields**, not proto extensions. They work perfectly with prost.

### ✅ Dynamic Extension Fields
```protobuf
google.protobuf.Value pbs = 4;  // ✅ PRESERVED
```

Fields typed as `google.protobuf.Value` or `google.protobuf.Struct` allow arbitrary JSON-like data and work with prost.

## Limitations

### ❌ Unknown Extension Fields

**Prost discards unknown fields** that aren't defined in the schema.

**Example:**
```json
{
  "source": {
    "ext": {
      "schain": {...},        // ✅ Parsed (defined in schema)
      "omidpn": "Inmobi",     // ✅ Parsed (defined in schema)
      "future_field_2026": 1  // ❌ LOST (not in schema)
    }
  }
}
```

**Impact:** If exchanges send non-standard ext fields, they will be silently dropped.

**Mitigation:** OpenRTB exchanges typically follow the spec. If this becomes an issue, we can:
1. Add catch-all fields to the schema
2. Switch to `rust-protobuf` (supports unknown field preservation)

## Migration Path

When prost adds editions support (issue #1031):

1. Remove patching logic from `build.rs`
2. Use the original proto file directly
3. Delete this document

## Verification

We manually verified that:
- ✅ No `extend` statements exist in `openrtb.proto`
- ✅ All commonly-used ext fields (schain, omidpn, omidpv, eids) are regular fields
- ✅ Extension ranges are empty placeholders

## Testing Compatibility

To verify OpenRTB compatibility:
1. Send real bid requests through the library
2. Check that all expected ext fields are accessible
3. Monitor for missing data that might indicate unknown fields

If unknown ext fields become critical, file an issue to discuss switching to `rust-protobuf`.

---

**Last Updated:** October 2025
**Prost Version:** 0.14
**OpenRTB Proto:** 2.6 (edition 2023 branch)
