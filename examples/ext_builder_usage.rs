use rtb::bid_request::{ImpBuilder, imp};
use rtb::extensions::ExtWithCustom;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Three Ways to Set Custom Ext Fields ===\n");

    // ===== Option 1: Builder-style with ExtWithCustom =====
    println!("Option 1: Builder-style (Recommended)");

    let imp_ext = ExtWithCustom::new(imp::Ext::default())
        .with_bool("force_bid".to_string(), true)
        .with_i64("channel".to_string(), 42)
        .with_string("gpid".to_string(), "/homepage/banner".to_string());

    let imp = ImpBuilder::default()
        .id("imp-001".to_string())
        .ext(imp_ext)
        .build()?;

    println!("  Imp ID: {}", imp.id);
    if let Some(ext) = &imp.ext {
        println!("  GPID (proto): {}", ext.gpid);
        println!(
            "  force_bid (custom): {:?}",
            ext.custom().get_bool("force_bid")
        );
        println!("  channel (custom): {:?}", ext.custom().get_i64("channel"));
    }

    // ===== Option 2: Mutable access to existing ext =====
    println!("\nOption 2: Mutable access to existing impression");

    let mut imp2 = ImpBuilder::default().id("imp-002".to_string()).build()?;

    // Initialize ext if it doesn't exist
    if imp2.ext.is_none() {
        imp2.ext = Some(ExtWithCustom::new(imp::Ext::default()));
    }

    // Add custom fields
    if let Some(ext) = imp2.ext.as_mut() {
        ext.custom_mut().insert_bool("force_bid".to_string(), true);
        ext.custom_mut().insert_i64("priority".to_string(), 10);
        ext.proto_mut().gpid = "/app/interstitial".to_string();
    }

    println!("  Imp ID: {}", imp2.id);
    if let Some(ext) = &imp2.ext {
        println!("  GPID (proto): {}", ext.gpid);
        println!(
            "  force_bid (custom): {:?}",
            ext.custom().get_bool("force_bid")
        );
        println!(
            "  priority (custom): {:?}",
            ext.custom().get_i64("priority")
        );
    }

    // ===== Option 3: From JSON (for complex scenarios) =====
    println!("\nOption 3: From JSON");

    let imp_ext_json: ExtWithCustom<imp::Ext> = serde_json::from_value(serde_json::json!({
        "gpid": "/video/preroll",
        "skadn": null,
        "force_bid": true,
        "custom_targeting": {
            "age": "18-24",
            "interests": ["gaming", "tech"]
        }
    }))?;

    let imp3 = ImpBuilder::default()
        .id("imp-003".to_string())
        .ext(imp_ext_json)
        .build()?;

    println!("  Imp ID: {}", imp3.id);
    if let Some(ext) = &imp3.ext {
        println!("  GPID (proto): {}", ext.gpid);
        println!(
            "  force_bid (custom): {:?}",
            ext.custom().get_bool("force_bid")
        );

        if let Some(targeting) = ext.custom().get_nested("custom_targeting") {
            println!("  Targeting age: {:?}", targeting.get_str("age"));
            if let Ok(Some(interests)) = targeting.get_array_as::<String>("interests") {
                println!("  Targeting interests: {:?}", interests);
            }
        }
    }

    println!("\n=== Serializing to JSON ===\n");
    let json = serde_json::to_string_pretty(&imp3)?;
    println!("{}", json);

    Ok(())
}
