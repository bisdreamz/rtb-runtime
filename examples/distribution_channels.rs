/// Example showing how to use app, site, and dooh fields in BidRequest
/// These are represented as a oneof enum since they're mutually exclusive
use openrtb_rs::openrtb::*;

fn main() {
    // Example 1: BidRequest with App
    let app_request = BidRequest {
        id: "request-with-app".to_string(),
        imp: vec![bid_request::Imp {
            id: "imp-1".to_string(),
            ..Default::default()
        }],
        distributionchannel_oneof: Some(bid_request::DistributionchannelOneof::App(
            bid_request::App {
                id: "app-123".to_string(),
                name: "My Awesome App".to_string(),
                bundle: "com.example.app".to_string(),
                storeurl: "https://play.google.com/store/apps/details?id=com.example.app".to_string(),
                ver: "1.2.3".to_string(),
                ..Default::default()
            }
        )),
        ..Default::default()
    };

    // Example 2: BidRequest with Site
    let site_request = BidRequest {
        id: "request-with-site".to_string(),
        imp: vec![bid_request::Imp {
            id: "imp-1".to_string(),
            ..Default::default()
        }],
        distributionchannel_oneof: Some(bid_request::DistributionchannelOneof::Site(
            bid_request::Site {
                id: "site-456".to_string(),
                name: "Example News".to_string(),
                domain: "news.example.com".to_string(),
                page: "https://news.example.com/article/123".to_string(),
                ..Default::default()
            }
        )),
        ..Default::default()
    };

    // Example 3: BidRequest with DOOH
    let dooh_request = BidRequest {
        id: "request-with-dooh".to_string(),
        imp: vec![bid_request::Imp {
            id: "imp-1".to_string(),
            ..Default::default()
        }],
        distributionchannel_oneof: Some(bid_request::DistributionchannelOneof::Dooh(
            bid_request::Dooh {
                id: "dooh-789".to_string(),
                name: "Times Square Billboard".to_string(),
                ..Default::default()
            }
        )),
        ..Default::default()
    };

    // Pattern matching to access the distribution channel
    println!("=== Example 1: App Request ===");
    match &app_request.distributionchannel_oneof {
        Some(bid_request::DistributionchannelOneof::App(app)) => {
            println!("Distribution Channel: App");
            println!("  App ID: {}", app.id);
            println!("  App Name: {}", app.name);
            println!("  Bundle: {}", app.bundle);
        }
        Some(bid_request::DistributionchannelOneof::Site(site)) => {
            println!("Distribution Channel: Site");
            println!("  Site ID: {}", site.id);
        }
        Some(bid_request::DistributionchannelOneof::Dooh(dooh)) => {
            println!("Distribution Channel: DOOH");
            println!("  DOOH ID: {}", dooh.id);
        }
        None => println!("No distribution channel specified"),
    }

    println!("\n=== Example 2: Site Request ===");
    match &site_request.distributionchannel_oneof {
        Some(bid_request::DistributionchannelOneof::Site(site)) => {
            println!("Distribution Channel: Site");
            println!("  Site ID: {}", site.id);
            println!("  Site Name: {}", site.name);
            println!("  Domain: {}", site.domain);
            println!("  Page: {}", site.page);
        }
        _ => println!("Not a site request"),
    }

    println!("\n=== Example 3: DOOH Request ===");
    match &dooh_request.distributionchannel_oneof {
        Some(bid_request::DistributionchannelOneof::Dooh(dooh)) => {
            println!("Distribution Channel: DOOH");
            println!("  DOOH ID: {}", dooh.id);
            println!("  DOOH Name: {}", dooh.name);
        }
        _ => println!("Not a DOOH request"),
    }

    println!("\n✅ All distribution channel types (app, site, dooh) are present!");
    println!("   They're represented as a 'oneof' enum because they're mutually exclusive.");
}
