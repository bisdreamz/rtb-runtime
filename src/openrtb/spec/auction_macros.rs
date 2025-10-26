/// ID of the bid request; from `BidRequest.id` attribute.
pub const AUCTION_ID: &str = "${AUCTION_ID}";

/// ID of the bid; from `BidResponse.bidid` attribute.
pub const AUCTION_BID_ID: &str = "${AUCTION_BID_ID}";

/// ID of the impression just won; from `imp.id` attribute.
pub const AUCTION_IMP_ID: &str = "${AUCTION_IMP_ID}";

/// ID of the bidder seat for whom the bid was made.
pub const AUCTION_SEAT_ID: &str = "${AUCTION_SEAT_ID}";

/// ID of the ad markup the bidder wishes to serve; from `bid.adid` attribute.
pub const AUCTION_AD_ID: &str = "${AUCTION_AD_ID}";

/// Clearing price using the same currency and units as the bid.
pub const AUCTION_PRICE: &str = "${AUCTION_PRICE}";

/// The currency used in the bid (explicit or implied); for confirmation only.
pub const AUCTION_CURRENCY: &str = "${AUCTION_CURRENCY}";

/// Market Bid Ratio defined as: clearance price / bid price.
pub const AUCTION_MBR: &str = "${AUCTION_MBR}";

/// Loss reason codes. Refer to List: Loss Reason Codes in OpenRTB 3.0.
pub const AUCTION_LOSS: &str = "${AUCTION_LOSS}";

/// Minimum bid to win the exchange's auction, using the same currency and units as the bid.
pub const AUCTION_MIN_TO_WIN: &str = "${AUCTION_MIN_TO_WIN}";

/// The total quantity of impressions won; for confirmation only.
/// This should always be less than or equal to the multiplier value sent in the bid request.
/// This value is a float value greater than zero and may be less than one.
/// Should be used to confirm that the buyer expects and understands the multiplier value.
pub const AUCTION_MULTIPLIER: &str = "${AUCTION_MULTIPLIER}";

/// Timestamp when the impression was fulfilled (e.g. when the ad is displayed) in Unix format
/// (i.e., milliseconds since the epoch). This may be used by platforms that cannot fire a
/// notification as soon as the impression takes place. If omitted, it is assumed the impression
/// took place a few seconds before the notification is fired.
pub const AUCTION_IMP_TS: &str = "${AUCTION_IMP_TS}";
