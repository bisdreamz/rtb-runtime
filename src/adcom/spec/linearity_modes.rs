//! Linearity Modes
//!
//! Options for media linearity (typically video). This corresponds to the required type of VAST response,
//! where a linear response is VAST containing video assets, and non-linear is a VAST response (typically) containing a banner/overlay.

use crate::spec_list;

spec_list! {
    /// Linear
    LINEAR = 1 => "Linear",

    /// Non-Linear (i.e., Overlay)
    NON_LINEAR = 2 => "Non-Linear (i.e., Overlay)",
}
