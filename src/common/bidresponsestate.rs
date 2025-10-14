use crate::BidResponse;

/// Standard enum for representing the state of a bidresponse after
/// a request has completed evaluation
pub enum BidResponseState {
    /// Indicates one or more valid bids are present
    Bid (BidResponse),
    /// Indicates no bids present for auction with the associated reason
    /// and optional detail message
    NoBid{ nbr: u32, desc: Option<String> },
}

impl From<BidResponseState> for BidResponse {
    fn from(value: BidResponseState) -> Self {
        match value {
            BidResponseState::Bid (b) => b,
            BidResponseState::NoBid{ nbr, .. } => {
                BidResponse {
                    nbr: nbr as i32,
                    ..Default::default()
                }
            }
        }
    }
}