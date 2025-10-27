//! No-Bid Reason Codes
//!
//! Options for a bidder to signal the exchange as to why it did not offer a bid for the item.

use crate::spec_list;

spec_list! {
    /// Unknown Error
    UNKNOWN_ERROR = 0 => "Unknown Error",

    /// Technical Error
    TECHNICAL_ERROR = 1 => "Technical Error",

    /// Invalid Request
    INVALID_REQUEST = 2 => "Invalid Request",

    /// Known Web Crawler
    KNOWN_WEB_CRAWLER = 3 => "Known Web Crawler",

    /// Suspected Non-Human Traffic
    SUSPECTED_NON_HUMAN_TRAFFIC = 4 => "Suspected Non-Human Traffic",

    /// Cloud, Data Center, or Proxy IP
    CLOUD_DATACENTER_PROXY_IP = 5 => "Cloud, Data Center, or Proxy IP",

    /// Unsupported Device
    UNSUPPORTED_DEVICE = 6 => "Unsupported Device",

    /// Blocked Publisher or Site
    BLOCKED_PUBLISHER_OR_SITE = 7 => "Blocked Publisher or Site",

    /// Unmatched User
    UNMATCHED_USER = 8 => "Unmatched User",

    /// Daily User Cap Met
    DAILY_USER_CAP_MET = 9 => "Daily User Cap Met",

    /// Daily Domain Cap Met
    DAILY_DOMAIN_CAP_MET = 10 => "Daily Domain Cap Met",

    /// Ads.txt Authorization Unavailable
    ADS_TXT_AUTHORIZATION_UNAVAILABLE = 11 => "Ads.txt Authorization Unavailable",

    /// Ads.txt Authorization Violation
    ADS_TXT_AUTHORIZATION_VIOLATION = 12 => "Ads.txt Authorization Violation",

    /// Ads.cert Authentication Unavailable
    ADS_CERT_AUTHENTICATION_UNAVAILABLE = 13 => "Ads.cert Authentication Unavailable",

    /// Ads.cert Authentication Violation
    ADS_CERT_AUTHENTICATION_VIOLATION = 14 => "Ads.cert Authentication Violation",

    /// Insufficient Auction Time
    INSUFFICIENT_AUCTION_TIME = 15 => "Insufficient Auction Time",

    /// Incomplete SupplyChain
    INCOMPLETE_SUPPLYCHAIN = 16 => "Incomplete SupplyChain",

    /// Blocked SupplyChain Node
    BLOCKED_SUPPLYCHAIN_NODE = 17 => "Blocked SupplyChain Node",
}
