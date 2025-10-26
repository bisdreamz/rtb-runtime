//! Display Context Types
//!
//! Types of context in which a native ad may appear (i.e., the type of content surrounding the ad on the page).
//! This is intended to denote primary content although other content may also appear on the page.
//! Note that there are two levels of detail grouped by 10s (i.e., 12 is a refined case of 100).

use crate::spec_list;

spec_list! {
    /// Content-centric context (e.g., newsfeed, article, image gallery, video gallery, etc.).
    CONTENT_CENTRIC = 10 => "Content-centric context",

    /// Primarily article content, which could include images, etc. as part of the article.
    ARTICLE = 11 => "Article content",

    /// Primarily video content.
    VIDEO = 12 => "Video content",

    /// Primarily audio content.
    AUDIO = 13 => "Audio content",

    /// Primarily image content.
    IMAGE = 14 => "Image content",

    /// User-generated content (e.g., forums, comments, etc.).
    USER_GENERATED = 15 => "User-generated content",

    /// Social-centric context (e.g., social network feed, email, chat, etc.).
    SOCIAL_CENTRIC = 20 => "Social-centric context",

    /// Primarily email content.
    EMAIL = 21 => "Email content",

    /// Primarily chat/IM content.
    CHAT = 22 => "Chat/IM content",

    /// Product context (e.g., product listings, details, recommendations, reviews, etc.).
    PRODUCT = 30 => "Product context",

    /// App store/marketplace.
    APP_STORE = 31 => "App store/marketplace",

    /// Product reviews site primarily, which may sell product secondarily.
    PRODUCT_REVIEWS = 32 => "Product reviews site",
}
