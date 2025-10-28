pub mod spec;
pub mod utils;

include!(concat!(env!("OUT_DIR"), "/com.iabtechlab.openrtb.v2.rs"));

include!(concat!(
    env!("OUT_DIR"),
    "/com.iabtechlab.openrtb.v2.serde.rs"
));
