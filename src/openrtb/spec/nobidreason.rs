#![allow(dead_code)]

/// Reserved and unused
pub const NONE: u32 = 0;
/// Technical error prevented a valid bid response
pub const TECHNICAL_ERROR: u32 = 1;
/// Request invalid json or missing required fields
pub const INVALID_REQUEST: u32 = 2;
/// Web crawler
pub const WEB_SPIDER: u32 = 3;
/// Suspected bot or IVT
pub const NONHUMAN_TRAFFIC: u32 = 4;
/// Datacenter or proxy IP address
pub const DC_PROXY_IP: u32 = 5;
/// Unsupported device type e.g. DOOH
pub const UNSUPPORTED_DEV: u32 = 6;
/// Blocked publisher or specific app/site domain
pub const BLOCKED_PUB_OR_SITE: u32 = 7;
/// Unknown or unmatched user, e.g. missing buyeruid
pub const UNKNOWN_USER: u32 = 8;
/// Daily frequency cap met for user
pub const DAILY_READER_CAP: u32 = 9;
/// Daily frequency cap met for domain
pub const DAILY_DOMAIN_CAP: u32 = 10;
