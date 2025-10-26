//! Pod Sequence
//!
//! Values for the pod sequence field, for use in audio and video content streams with one or more ad pods.

use crate::spec_list_i32;

spec_list_i32! {
    /// Last pod in the content stream
    LAST = -1 => "Last pod in the content stream",

    /// Any pod in the content stream
    ANY = 0 => "Any pod in the content stream",

    /// First pod in the content stream
    FIRST = 1 => "First pod in the content stream",
}
