//! DOOH Venue Taxonomies
//!
//! A list of supported taxonomies describing the locations and contexts in which Out-Of-Home media may be experienced.
//! Taxonomies entries are expected to refer to a specific version, unless a given taxonomy has explicit semantics for forward compatibility and handling updates.

use crate::spec_list;

spec_list! {
    /// AdCom DOOH Venue Types (deprecated)
    ADCOM_DOOH_VENUE_TYPES = 0 => "AdCom DOOH Venue Types (deprecated)",

    /// OpenOOH Venue Taxonomy 1.0
    OPENOOH_VENUE_TAXONOMY_1_0 = 1 => "OpenOOH Venue Taxonomy 1.0",

    /// DPAA Device Venue Types
    DPAA_DEVICE_VENUE_TYPES = 2 => "DPAA Device Venue Types",

    /// DMI Categorization of Venues 1.1
    DMI_CATEGORIZATION_1_1 = 3 => "DMI Categorization of Venues 1.1",

    /// OMA taxonomy Jan 2022
    OMA_TAXONOMY_JAN_2022 = 4 => "OMA taxonomy Jan 2022",

    /// OpenOOH Venue Taxonomy 1.1
    OPENOOH_VENUE_TAXONOMY_1_1 = 5 => "OpenOOH Venue Taxonomy 1.1",
}
