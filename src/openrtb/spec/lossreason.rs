//! Loss Reason Codes
//!
//! Options for an exchange to inform a bidder as to the reason why they did not win an item.

use crate::spec_list;

spec_list! {
    /// Bid Won
    BID_WON = 0 => "Bid Won",

    /// Internal Error
    INTERNAL_ERROR = 1 => "Internal Error",

    /// Impression Opportunity Expired
    IMPRESSION_OPPORTUNITY_EXPIRED = 2 => "Impression Opportunity Expired",

    /// Invalid Bid Response
    INVALID_BID_RESPONSE = 3 => "Invalid Bid Response",

    /// Invalid Deal ID
    INVALID_DEAL_ID = 4 => "Invalid Deal ID",

    /// Invalid Auction ID
    INVALID_AUCTION_ID = 5 => "Invalid Auction ID",

    /// Invalid Advertiser Domain
    INVALID_ADVERTISER_DOMAIN = 6 => "Invalid Advertiser Domain",

    /// Missing Markup
    MISSING_MARKUP = 7 => "Missing Markup",

    /// Missing Creative ID
    MISSING_CREATIVE_ID = 8 => "Missing Creative ID",

    /// Missing Bid Price
    MISSING_BID_PRICE = 9 => "Missing Bid Price",

    /// Missing Minimum Creative Approval Data
    MISSING_MINIMUM_CREATIVE_APPROVAL_DATA = 10 => "Missing Minimum Creative Approval Data",

    /// Bid was Below Auction Floor
    BID_BELOW_AUCTION_FLOOR = 100 => "Bid was Below Auction Floor",

    /// Bid was Below Deal Floor
    BID_BELOW_DEAL_FLOOR = 101 => "Bid was Below Deal Floor",

    /// Lost to Higher Bid
    LOST_TO_HIGHER_BID = 102 => "Lost to Higher Bid",

    /// Lost to a Bid for a Deal
    LOST_TO_BID_FOR_DEAL = 103 => "Lost to a Bid for a Deal",

    /// Buyer Seat Blocked
    BUYER_SEAT_BLOCKED = 104 => "Buyer Seat Blocked",

    /// Creative Filtered - General; Reason Unknown
    CREATIVE_FILTERED_GENERAL = 200 => "Creative Filtered - General; Reason Unknown",

    /// Creative Filtered - Pending Processing by Exchange (e.g., approval, transcoding, etc.)
    CREATIVE_FILTERED_PENDING_PROCESSING = 201 => "Creative Filtered - Pending Processing by Exchange",

    /// Creative Filtered - Disapproved by Exchange
    CREATIVE_FILTERED_DISAPPROVED = 202 => "Creative Filtered - Disapproved by Exchange",

    /// Creative Filtered - Size Not Allowed
    CREATIVE_FILTERED_SIZE_NOT_ALLOWED = 203 => "Creative Filtered - Size Not Allowed",

    /// Creative Filtered - Incorrect Creative Format
    CREATIVE_FILTERED_INCORRECT_FORMAT = 204 => "Creative Filtered - Incorrect Creative Format",

    /// Creative Filtered - Advertiser Exclusions
    CREATIVE_FILTERED_ADVERTISER_EXCLUSIONS = 205 => "Creative Filtered - Advertiser Exclusions",

    /// Creative Filtered - App Store ID Exclusions
    CREATIVE_FILTERED_APP_STORE_ID_EXCLUSIONS = 206 => "Creative Filtered - App Store ID Exclusions",

    /// Creative Filtered - Not Secure
    CREATIVE_FILTERED_NOT_SECURE = 207 => "Creative Filtered - Not Secure",

    /// Creative Filtered - Language Exclusions
    CREATIVE_FILTERED_LANGUAGE_EXCLUSIONS = 208 => "Creative Filtered - Language Exclusions",

    /// Creative Filtered - Category Exclusions
    CREATIVE_FILTERED_CATEGORY_EXCLUSIONS = 209 => "Creative Filtered - Category Exclusions",

    /// Creative Filtered - Creative Attribute Exclusions
    CREATIVE_FILTERED_CREATIVE_ATTRIBUTE_EXCLUSIONS = 210 => "Creative Filtered - Creative Attribute Exclusions",

    /// Creative Filtered - Ad Type Exclusions
    CREATIVE_FILTERED_AD_TYPE_EXCLUSIONS = 211 => "Creative Filtered - Ad Type Exclusions",

    /// Creative Filtered - Animation Too Long
    CREATIVE_FILTERED_ANIMATION_TOO_LONG = 212 => "Creative Filtered - Animation Too Long",

    /// Creative Filtered - Not Allowed in Deal
    CREATIVE_FILTERED_NOT_ALLOWED_IN_DEAL = 213 => "Creative Filtered - Not Allowed in Deal",

    /// Creative Filtered - Invalid SKAdNetwork
    CREATIVE_FILTERED_INVALID_SKADNETWORK = 214 => "Creative Filtered - Invalid SKAdNetwork",

    /// Creative Filtered - App Bundle Exclusions
    CREATIVE_FILTERED_APP_BUNDLE_EXCLUSIONS = 215 => "Creative Filtered - App Bundle Exclusions",
}
