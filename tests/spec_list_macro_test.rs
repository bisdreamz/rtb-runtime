use rtb::spec::adcom::devicetype;

#[test]
fn test_devicetype_constants() {
    assert_eq!(devicetype::PHONE, 4);
    assert_eq!(devicetype::TABLET, 5);
    assert_eq!(devicetype::CONNECTED_TV, 3);
}

#[test]
fn test_devicetype_name() {
    assert_eq!(devicetype::name(4), Some("PHONE"));
    assert_eq!(devicetype::name(5), Some("TABLET"));
    assert_eq!(devicetype::name(999), None);
}

#[test]
fn test_devicetype_description() {
    assert_eq!(devicetype::description(4), Some("Phone"));
    assert_eq!(devicetype::description(5), Some("Tablet"));
    assert_eq!(devicetype::description(3), Some("Connected TV"));
    assert_eq!(devicetype::description(999), None);
}

#[test]
fn test_devicetype_is_valid() {
    assert!(devicetype::is_valid(1));
    assert!(devicetype::is_valid(4));
    assert!(devicetype::is_valid(8));
    assert!(!devicetype::is_valid(0));
    assert!(!devicetype::is_valid(999));
}

#[test]
fn test_devicetype_all_values() {
    let values = devicetype::all_values();
    assert_eq!(values.len(), 8);
    assert!(values.contains(&4));
    assert!(values.contains(&5));
}
