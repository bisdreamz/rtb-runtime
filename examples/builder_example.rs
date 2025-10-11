use openrtb_rs::bid_request;
use openrtb_rs::openrtb::*;

fn main() {
    // Create a BidRequest using the builder pattern
    let request = BidRequestBuilder::default()
        .id("auction-123".to_string())
        .test(false)
        .at(2) // Second price auction
        .tmax(100)
        .imp(vec![
            bid_request::ImpBuilder::default()
                .id("impression-1".to_string())
                .bidfloor(0.5)
                .bidfloorcur("USD".to_string())
                .build()
                .unwrap(),
        ])
        .badv(vec!["spam.com".to_string(), "malware.org".to_string()])
        .build()
        .unwrap();

    println!("Created BidRequest with ID: {}", request.id);
    println!("  Test mode: {}", request.test);
    println!("  Timeout: {}ms", request.tmax);
    println!("  Impressions: {}", request.imp.len());
    println!("  Blocked advertisers: {:?}", request.badv);

    // Create a Device using the builder
    let device = bid_request::DeviceBuilder::default()
        .ua("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)".to_string())
        .ip("192.168.1.1".to_string())
        .devicetype(1) // Mobile/Tablet
        .make("Apple".to_string())
        .model("iPhone 12".to_string())
        .os("iOS".to_string())
        .osv("14.0".to_string())
        .build()
        .unwrap();

    println!("\nCreated Device:");
    println!("  User Agent: {}", device.ua);
    println!("  IP: {}", device.ip);
    println!("  Make/Model: {} {}", device.make, device.model);
    println!("  OS: {} {}", device.os, device.osv);

    // Create a User using the builder
    let user = bid_request::UserBuilder::default()
        .id("user-abc123".to_string())
        .buyeruid("buyer-456".to_string())
        .yob(1990)
        .gender("M".to_string())
        .build()
        .unwrap();

    println!("\nCreated User:");
    println!("  User ID: {}", user.id);
    println!("  Buyer UID: {}", user.buyeruid);
    println!("  Year of Birth: {}", user.yob);
    println!("  Gender: {}", user.gender);

    // Create a complete request with all components
    let complete_request = BidRequestBuilder::default()
        .id("auction-789".to_string())
        .imp(vec![
            bid_request::ImpBuilder::default()
                .id("imp-001".to_string())
                .tagid("ad-slot-1".to_string())
                .bidfloor(1.0)
                .bidfloorcur("USD".to_string())
                .secure(true) // HTTPS required
                .build()
                .unwrap(),
        ])
        .device(device)
        .user(user)
        .cur(vec!["USD".to_string()])
        .build()
        .unwrap();

    println!("\nCreated complete BidRequest:");
    println!("  Request ID: {}", complete_request.id);
    println!("  Impressions: {}", complete_request.imp.len());
    println!("  Accepted currencies: {:?}", complete_request.cur);
    if let Some(device) = &complete_request.device {
        println!("  Has device info: Yes ({})", device.make);
    }
    if let Some(user) = &complete_request.user {
        println!("  Has user info: Yes ({})", user.id);
    }
}
