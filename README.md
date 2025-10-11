# openrtb-rs

Rust bindings for OpenRTB 2.x protocol with automatic JSON handling.

## Features

- Generated from official OpenRTB protobuf definitions
- Standard `serde_json` for parsing and serialization
- Automatic boolean field handling (0/1 integers per OpenRTB spec)
- Type-safe extension field support for custom fields
- Builder pattern for convenient struct construction
- Full gRPC support via tonic

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
openrtb-rs = "0.1"
serde_json = "1.0"
```

### Basic Usage

```rust
use openrtb_rs::BidRequest;

// Parse OpenRTB JSON
let request: BidRequest = serde_json::from_str(json)?;

// Access fields
println!("Request ID: {}", request.id);
println!("Test mode: {}", request.test);

// Serialize back
let json = serde_json::to_string(&request)?;
```

That's it! Boolean fields automatically serialize as 0/1 integers per the OpenRTB spec.

### Working with Extension Fields

OpenRTB allows custom fields in `ext` objects. Access both proto-defined and custom fields easily:

```rust
use openrtb_rs::BidRequest;

let request: BidRequest = serde_json::from_str(json)?;

if let Some(imp) = request.imp.first() {
    if let Some(ref ext) = imp.ext {
        // Proto fields - full autocomplete support
        let gpid = &ext.gpid;
        let skadn = &ext.skadn;

        // Custom fields - dynamic access
        let channel = ext.custom().get_i64("channel");
        let enabled = ext.custom().get_bool("enabled");

        // With defaults
        let channel = ext.custom().get_i64_or("channel", 0);
    }
}
```

See [examples/](examples/) for more detailed usage patterns.

## Boolean Field Handling

OpenRTB specifies that certain boolean fields must be serialized as integers (0 or 1) in JSON, not as `true`/`false`. This library handles this automatically at the serde level.

Fields affected include: `test`, `allimps`, `secure`, `instl`, `skip`, `topframe`, `dnt`, `lmt`, `coppa`, `gdpr`, and many others.

**Input**: Accepts both `true`/`false` and `0`/`1` for maximum compatibility
**Output**: Always outputs `0`/`1` for OpenRTB compliance

## Extension Fields

The OpenRTB protobuf definitions use `extensions` declarations to allow custom fields beyond the standard specification. Since prost doesn't support extensions yet, this library provides a compatibility layer.

### Accessing Proto Fields

Proto-defined fields are accessible directly with full IDE support:

```rust
let gpid = imp.ext.gpid;              // String field
let skadn = imp.ext.skadn;            // Complex nested object
let intrinsic = imp.ext.intrinsic;    // Boolean field
```

### Accessing Custom Fields

Custom/unknown fields are accessed via the `.custom()` method:

```rust
// Primitive types
let channel = ext.custom().get_i64("channel");
let price = ext.custom().get_f64("bidfloor");
let name = ext.custom().get_str("name");

// With defaults
let enabled = ext.custom().get_bool_or("enabled", true);

// Collections
let categories = ext.custom().get_array("categories");

// Nested objects
if let Some(metadata) = ext.custom().get_nested("metadata") {
    let version = metadata.get_str("version");
}
```

### Type-Safe Custom Extensions

When you know the structure of custom fields, deserialize into a typed struct:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct VideoExt {
    channel: i64,
    rewarded: bool,
    categories: Vec<String>,
}

let typed: VideoExt = ext.custom().as_typed()?;
println!("Channel: {}", typed.channel);
```

## gRPC and Protobuf Support

This library provides full support for both JSON (HTTP) and Protobuf (gRPC) with a unified API. The same `BidRequest` type works seamlessly with both transports.

### Unified API

Write your business logic once and it works with both JSON and Protobuf:

```rust
use openrtb_rs::BidRequest;
use prost::Message;

// Helper function works regardless of how the request was received
fn process_request(request: &BidRequest) -> Option<i64> {
    request.imp.first()?
        .ext.as_ref()?
        .custom()
        .get_i64("channel")
}

// From JSON (e.g., actix-web)
let json_request: BidRequest = serde_json::from_str(json_body)?;
let channel = process_request(&json_request);

// From Protobuf (e.g., tonic gRPC)
let proto_request: BidRequest = BidRequest::decode(grpc_bytes)?;
let channel = process_request(&proto_request);
```

### Extension Field Behavior

Extension fields behave differently depending on the transport:

**JSON (via serde):**
- Proto-defined fields: ✓ Preserved
- Custom fields: ✓ Captured and accessible via `.custom()`

**Protobuf (via prost):**
- Proto-defined fields: ✓ Preserved
- Custom fields: ✗ Not preserved (don't exist in proto schema)

This is expected behavior. Custom fields are specific to JSON exchanges and will be empty after protobuf encoding/decoding:

```rust
// Parse from JSON
let json_request: BidRequest = serde_json::from_str(r#"{
    "id": "test",
    "imp": [{
        "id": "1",
        "ext": {
            "gpid": "/homepage/banner",
            "channel": 42
        }
    }]
}"#)?;

// Both proto and custom fields available
assert_eq!(json_request.imp[0].ext.as_ref().unwrap().gpid, "/homepage/banner");
assert_eq!(json_request.imp[0].ext.as_ref().unwrap().custom().get_i64("channel"), Some(42));

// Encode to protobuf and back
let mut buf = Vec::new();
json_request.encode(&mut buf)?;
let proto_request = BidRequest::decode(&buf[..])?;

// Only proto-defined fields survive
assert_eq!(proto_request.imp[0].ext.as_ref().unwrap().gpid, "/homepage/banner");
assert_eq!(proto_request.imp[0].ext.as_ref().unwrap().custom().get_i64("channel"), None);
```

### Using with tonic

Add tonic to your dependencies:

```toml
[dependencies]
openrtb-rs = "0.1"
tonic = "0.14"
prost = "0.14"
```

Define your gRPC service:

```rust
use openrtb_rs::{BidRequest, BidResponse};
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl BidService for MyBidder {
    async fn bid(
        &self,
        request: Request<BidRequest>,
    ) -> Result<Response<BidResponse>, Status> {
        let bid_request = request.into_inner();

        // Access fields normally
        let request_id = &bid_request.id;

        // Process extensions (proto fields only)
        if let Some(imp) = bid_request.imp.first() {
            if let Some(ref ext) = imp.ext {
                let gpid = &ext.gpid; // Proto-defined field
                // Custom fields will be empty in protobuf
            }
        }

        // Build response
        let response = BidResponse::default();
        Ok(Response::new(response))
    }
}
```

## Building

```bash
# Initialize git submodule for OpenRTB proto definitions
git submodule update --init

# Build
cargo build
```

## Examples

Run the included examples to see the library in action:

```bash
# Basic usage
cargo run --example basic_usage

# Custom extension types
cargo run --example custom_extensions
```

## Performance

Benchmarks on typical hardware show:

- **Parsing**: 159K requests/second
- **Serialization**: 306K requests/second
- **Full roundtrip**: 102K requests/second

The boolean conversion and extension field handling add negligible overhead compared to standard serde_json.

Run benchmarks:

```bash
cargo bench
```

## Architecture

### Build-Time Code Generation

The build process (see `build.rs`):

1. Patches OpenRTB proto from Protobuf Editions to proto3 (prost compatibility)
2. Generates Rust types with prost
3. Generates serde implementations with pbjson
4. Patches boolean fields to use custom serde wrappers
5. Patches extension fields to use `ExtWithCustom<T>` wrapper

This happens once at build time. The resulting code has zero runtime overhead.

### Runtime Compatibility Layer

The `compat/` module (internal) provides:

- `bool_as_int`: Serde helpers for boolean/integer conversion
- `extensions`: Type wrappers for extension field handling

See [src/compat/README.md](src/compat/README.md) for detailed internals documentation.

## Dependencies

- `prost 0.14` - Protobuf support
- `tonic 0.14` - gRPC framework
- `pbjson 0.8` - JSON serialization for protobuf
- `serde_json 1.0` - Standard JSON handling
- `derive_builder 0.20` - Builder pattern generation

## Contributing

This is a library focused on OpenRTB protocol support. Contributions should maintain:

- Zero-cost abstractions where possible
- Professional, production-ready code
- Comprehensive tests and documentation
- Compatibility with the OpenRTB specification

## License

This project uses the official OpenRTB protobuf definitions as a git submodule. The protobuf definitions are copyright IAB Tech Lab and licensed under Apache 2.0.

The Rust binding code in this repository is provided as-is for use with OpenRTB implementations.

## OpenRTB Version

This library targets OpenRTB 2.x as defined by the IAB Tech Lab protobuf definitions.

## Support

For issues with the library itself, please file a GitHub issue.

For questions about the OpenRTB specification, consult the [IAB Tech Lab documentation](https://github.com/InteractiveAdvertisingBureau/openrtb2.x).
