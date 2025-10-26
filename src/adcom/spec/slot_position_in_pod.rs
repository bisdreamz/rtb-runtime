//! Slot Position in Pod
//!
//! Values for the slot position in pod field, for use in audio and video ad pods.

use crate::spec_list_i32;

spec_list_i32! {
    /// Last ad in the pod
    LAST = -1 => "Last ad in the pod",

    /// Any ad in the pod
    ANY = 0 => "Any ad in the pod",

    /// First ad in the pod
    FIRST = 1 => "First ad in the pod",

    /// First or Last ad in the pod
    FIRST_OR_LAST = 2 => "First or Last ad in the pod",
}
