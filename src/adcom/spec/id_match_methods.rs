//! ID Match Methods
//!
//! Enumerations for various ways an ID could be matched to an ad request, and if they pertain to a single property or app.
//! Should be used in conjunction with the `mm` attribute in Object: EID of OpenRTB 2.x.

use crate::spec_list;

spec_list! {
    /// Unknown
    UNKNOWN = 0 => "Unknown",

    /// No Match: No matching has occurred. The associated ID came directly from a 3rd-party cookie or OS-provided resettable device ID for advertising (IFA).
    NO_MATCH = 1 => "No Match",

    /// Browser Cookie Sync: Real time cookie sync as described in Appendix: Cookie Based ID Syncing of OpenRTB 2.x
    BROWSER_COOKIE_SYNC = 2 => "Browser Cookie Sync",

    /// Authenticated: ID match was based on user authentication such as an email login or hashed PII
    AUTHENTICATED = 3 => "Authenticated",

    /// Observed: ID match was based on a 1st party observation, but without user authentication (e.g. GUID, SharedID, Session IDs, CHIPS or other 1st party cookies contained in localStorage)
    OBSERVED = 4 => "Observed",

    /// Inference: ID match was inferred from linkage based on non-authenticated features across multiple browsers or devices (e.g. IP address and/or UserAgent)
    INFERENCE = 5 => "Inference",
}
