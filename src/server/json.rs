use crate::BidResponse;
use crate::common::bidresponsestate::BidResponseState;
use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};

pub struct JsonBidResponseState(pub BidResponseState);

impl Responder for JsonBidResponseState {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            BidResponseState::Bid(bidresponse) => HttpResponse::Ok().json(bidresponse),
            BidResponseState::NoBidReason { reqid, nbr, desc } => HttpResponse::Ok()
                .reason(desc.unwrap_or("No Bid"))
                .json(BidResponse {
                    id: reqid,
                    nbr: nbr as i32,
                    ..Default::default()
                }),
            BidResponseState::NoBid { desc } => {
                let response = HttpResponse::NoContent()
                    .reason(desc.unwrap_or("No Bid"))
                    .finish();
                response
            }
        }
    }
}
