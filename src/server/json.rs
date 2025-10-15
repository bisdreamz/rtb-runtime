use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};
use crate::BidResponse;
use crate::common::bidresponsestate::BidResponseState;

pub struct JsonBidResponseState(pub BidResponseState);

impl Responder for JsonBidResponseState {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match Option::<BidResponse>::from(self.0) {
            Some(bid_response) => HttpResponse::Ok().json(bid_response),
            None => HttpResponse::NoContent().finish(),
        }
    }
}