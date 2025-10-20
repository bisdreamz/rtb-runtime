use crate::BidResponse;

/// Standard enum for representing the state of a bidresponse after
/// a request has completed evaluation
#[derive(Debug, Clone)]
pub enum BidResponseState {
    /// Indicates one or more valid bids are present
    ///
    /// # Arguments
    /// A bid response object which may contain either valid bids
    /// or a manually constructed nbr response
    ///
    /// # Behavior
    /// If returned as `JsonBidResponseState` or `Protobuf` to actix, will
    /// return an http 200 with the serialized bidresponse
    Bid (BidResponse),
    /// Indicates no bids present for auction with the associated reason
    /// and optional detail message. If paired with the actix server,
    /// will respond with an http200 plus nbr object.
    ///
    /// # Arguments
    /// * `reqid` - The id of the corresponding bidrequest
    /// * `nbr` - The nbr value to return. See [`crate::openrtb::spec::nobidreason`]
    /// * `desc` - An optional description for convenience
    ///
    /// # Behavior
    /// If returned as a `JsonBidResponseState` or `Protobuf` to actix,
    /// will return http 200 with the nbr object and the
    /// desc as the http status message if present
    NoBidReason { reqid: String, nbr: u32, desc: Option<&'static str> },
    /// Indicates no bids present. If paired with actix server,
    /// this will send an http 204
    ///
    /// # Arguments
    /// * `desc` - An optional description for convenience
    ///
    /// # Behavior
    /// If returned as `JsonBidResponseState` or `Protobuf` to actix,
    /// will return an http 204 with the desc as the
    /// http status message if present
    NoBid { desc: Option<&'static str> },
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