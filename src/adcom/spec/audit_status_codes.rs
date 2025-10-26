//! Audit Status Codes
//!
//! Codes used in Audit objects to reflect status or workflow state.

use crate::spec_list;

spec_list! {
    /// Pending Audit: An audit has not yet been completed on this ad.
    PENDING_AUDIT = 1 => "Pending Audit",

    /// Pre-Approved: An audit has not yet been completed on this ad. Subject to vendors' policies, it can be recommended for use.
    PRE_APPROVED = 2 => "Pre-Approved",

    /// Approved: The audit is complete and the ad is approved for use.
    APPROVED = 3 => "Approved",

    /// Denied: The audit is complete, but the ad has been found unacceptable in some material aspect and is disapproved for use.
    DENIED = 4 => "Denied",

    /// Changed; Resubmission Requested: A version of the ad has been detected in use that is materially different from the version that was previously audited.
    CHANGED_RESUBMIT = 5 => "Changed; Resubmission Requested",

    /// Expired: The ad has been marked as expired by the vendor.
    EXPIRED = 6 => "Expired",
}
