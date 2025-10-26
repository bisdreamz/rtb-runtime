//! Device Type enumeration from AdCom 1.0
//!
//! The general type of device. Refer to List: Device Types in AdCOM 1.0.
//! Used in OpenRTB 2.x `Device.devicetype` field.

use crate::spec_list;

spec_list! {
    /// Mobile/Tablet - General
    MOBILE_TABLET_GENERAL = 1 => "Mobile/Tablet - General",

    /// Personal Computer
    PERSONAL_COMPUTER = 2 => "Personal Computer",

    /// Connected TV
    CONNECTED_TV = 3 => "Connected TV",

    /// Phone
    PHONE = 4 => "Phone",

    /// Tablet
    TABLET = 5 => "Tablet",

    /// Connected Device
    CONNECTED_DEVICE = 6 => "Connected Device",

    /// Set Top Box
    SET_TOP_BOX = 7 => "Set Top Box",

    /// Digital Out-Of-Home (DOOH)
    DOOH = 8 => "DOOH",
}
