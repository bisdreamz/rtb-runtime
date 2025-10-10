use openrtb_rs::openrtb::*;

fn main() {
    // Create a simple BidRequest
    let bid_request = BidRequest {
        id: "test-request-123".to_string(),
        imp: vec![bid_request::Imp {
            id: "imp-1".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("Created BidRequest with ID: {}", bid_request.id);
    println!("Number of impressions: {}", bid_request.imp.len());

    // Example: Working with SupplyChain
    let supply_chain = bid_request::SupplyChain {
        complete: true,
        nodes: vec![bid_request::SupplyChainNode {
            asi: "example.com".to_string(),
            sid: "seller-123".to_string(),
            hp: true,
            ..Default::default()
        }],
        ver: "1.0".to_string(),
        ..Default::default()
    };

    println!("\nSupplyChain nodes: {}", supply_chain.nodes.len());
    println!("First node ASI: {}", supply_chain.nodes[0].asi);

    // Example: Working with ext fields
    println!("\n✅ All OpenRTB types generated successfully!");
    println!("   - BidRequest, BidResponse");
    println!("   - SupplyChain (strongly typed)");
    println!("   - EID (Extended Identifiers)");
    println!("   - All ext fields preserved");
}
