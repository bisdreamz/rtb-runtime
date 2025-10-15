use crate::BidResponse;

/// Standard enum for representing the state of a bidresponse after
/// a request has completed evaluation
pub enum BidResponseState {
    /// Indicates one or more valid bids are present
    Bid (BidResponse),
    /// Indicates no bids present for auction with the associated reason
    /// and optional detail message. If paired with the actix server,
    /// will respond with an http200 plus nbr object.
    NoBidReason { nbr: u32, desc: Option<String> },
    /// Indicates no bids present. If paired with actix server,
    /// this will send an http 204. Optionally attach a reason
    /// for logging.
    NoBid { desc: Option<String> },
}

impl From<BidResponseState> for Option<BidResponse> {
    fn from(value: BidResponseState) -> Self {
        match value {
            BidResponseState::Bid (b) => Some(b),
            BidResponseState::NoBidReason { nbr, .. } => {
                Some(BidResponse {
                    nbr: nbr as i32,
                    ..Default::default()
                })
            },
            BidResponseState::NoBid { .. } => None
        }
    }
}