use rtb::openrtb::spec::auction_macros;

fn main() {
    println!("=== OpenRTB 2.x Substitution Macros ===\n");

    println!("Win notice URL example:");
    let nurl = format!(
        "https://adserver.com/win?id={}&price={}&imp={}",
        auction_macros::AUCTION_ID,
        auction_macros::AUCTION_PRICE,
        auction_macros::AUCTION_IMP_ID
    );
    println!("  {}\n", nurl);

    println!("Billing notice URL example:");
    let burl = format!(
        "https://adserver.com/bill?id={}&price={}&multiplier={}",
        auction_macros::AUCTION_BID_ID,
        auction_macros::AUCTION_PRICE,
        auction_macros::AUCTION_MULTIPLIER
    );
    println!("  {}\n", burl);

    println!("Loss notice URL example:");
    let lurl = format!(
        "https://adserver.com/loss?id={}&reason={}&min_to_win={}",
        auction_macros::AUCTION_ID,
        auction_macros::AUCTION_LOSS,
        auction_macros::AUCTION_MIN_TO_WIN
    );
    println!("  {}\n", lurl);

    println!("Ad markup with tracking pixel:");
    let adm = format!(
        r#"<img src="https://adserver.com/track?id={}&imp={}&ts={}" />"#,
        auction_macros::AUCTION_AD_ID,
        auction_macros::AUCTION_IMP_ID,
        auction_macros::AUCTION_IMP_TS
    );
    println!("  {}\n", adm);

    println!("All available macros:");
    println!("  AUCTION_ID:         {}", auction_macros::AUCTION_ID);
    println!("  AUCTION_BID_ID:     {}", auction_macros::AUCTION_BID_ID);
    println!("  AUCTION_IMP_ID:     {}", auction_macros::AUCTION_IMP_ID);
    println!("  AUCTION_SEAT_ID:    {}", auction_macros::AUCTION_SEAT_ID);
    println!("  AUCTION_AD_ID:      {}", auction_macros::AUCTION_AD_ID);
    println!("  AUCTION_PRICE:      {}", auction_macros::AUCTION_PRICE);
    println!("  AUCTION_CURRENCY:   {}", auction_macros::AUCTION_CURRENCY);
    println!("  AUCTION_MBR:        {}", auction_macros::AUCTION_MBR);
    println!("  AUCTION_LOSS:       {}", auction_macros::AUCTION_LOSS);
    println!(
        "  AUCTION_MIN_TO_WIN: {}",
        auction_macros::AUCTION_MIN_TO_WIN
    );
    println!(
        "  AUCTION_MULTIPLIER: {}",
        auction_macros::AUCTION_MULTIPLIER
    );
    println!("  AUCTION_IMP_TS:     {}", auction_macros::AUCTION_IMP_TS);
}
