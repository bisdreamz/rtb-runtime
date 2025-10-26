//! Native Data Asset Types
//!
//! Common data asset types. This list is non-exhaustive and is intended to be expanded over time.
//! Size recommendations are noted as "maximum length of at least".

use crate::spec_list;

spec_list! {
    /// sponsored: "Sponsored By" message which should contain the brand name of the sponsor. Recommended maximum length of at least 25 characters.
    SPONSORED = 1 => "sponsored",

    /// desc: Descriptive text associated with the product or service being advertised. Long text lengths may be truncated or ellipsed when rendered. Recommended maximum length of at least 140 characters.
    DESC = 2 => "desc",

    /// rating: Numeric rating of the product (e.g., an app's rating). Recommended integer range of 0-5.
    RATING = 3 => "rating",

    /// likes: Number of social ratings or "likes" of the product.
    LIKES = 4 => "likes",

    /// downloads: Number downloads and/or installs of the product.
    DOWNLOADS = 5 => "downloads",

    /// price: Price of the product, app, or in-app purchase. Value should include currency symbol in localized format (e.g., "$10").
    PRICE = 6 => "price",

    /// saleprice: Sale price that can be used together with "price" to indicate a comparative discounted price. Value should include currency symbol in localized format (e.g., "$8.50").
    SALEPRICE = 7 => "saleprice",

    /// phone: A formatted phone number.
    PHONE = 8 => "phone",

    /// address: A formatted address.
    ADDRESS = 9 => "address",

    /// desc2: Additional descriptive text associated with the product.
    DESC2 = 10 => "desc2",

    /// displayurl: Display URL for the ad. To be used when sponsoring entity doesn't own the content (e.g., "Sponsored by Brand on Site", where Site is specified in this data asset).
    DISPLAYURL = 11 => "displayurl",

    /// ctatext: Description of the call to action (CTA) button for the destination URL. Recommended maximum length of at least 15 characters.
    CTATEXT = 12 => "ctatext",
}
