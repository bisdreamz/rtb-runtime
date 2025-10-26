//! Category Taxonomies
//!
//! Options for taxonomies that can be used to describe content, audience, and ad creative categories.

use crate::spec_list;

spec_list! {
    /// IAB Tech Lab Content Category Taxonomy 1.0: Deprecated, and recommend NOT be used since it does not have SCD flags.
    CONTENT_V1_0 = 1 => "IAB Tech Lab Content Category Taxonomy 1.0",

    /// IAB Tech Lab Content Category Taxonomy 2.0: Deprecated, and recommend NOT be used since it does not have SCD flags.
    CONTENT_V2_0 = 2 => "IAB Tech Lab Content Category Taxonomy 2.0",

    /// IAB Tech Lab Ad Product Taxonomy 1.0
    AD_PRODUCT_V1_0 = 3 => "IAB Tech Lab Ad Product Taxonomy 1.0",

    /// IAB Tech Lab Audience Taxonomy 1.1
    AUDIENCE_V1_1 = 4 => "IAB Tech Lab Audience Taxonomy 1.1",

    /// IAB Tech Lab Content Taxonomy 2.1
    CONTENT_V2_1 = 5 => "IAB Tech Lab Content Taxonomy 2.1",

    /// IAB Tech Lab Content Taxonomy 2.2
    CONTENT_V2_2 = 6 => "IAB Tech Lab Content Taxonomy 2.2",

    /// IAB Tech Lab Content Taxonomy 3.0
    CONTENT_V3_0 = 7 => "IAB Tech Lab Content Taxonomy 3.0",

    /// IAB Tech Lab Ad Product Taxonomy 2.0
    AD_PRODUCT_V2_0 = 8 => "IAB Tech Lab Ad Product Taxonomy 2.0",

    /// IAB Tech Lab Content Taxonomy 3.1
    CONTENT_V3_1 = 9 => "IAB Tech Lab Content Taxonomy 3.1",
}
