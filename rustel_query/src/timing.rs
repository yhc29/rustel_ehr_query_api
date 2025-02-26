use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};
use std::time::Instant;

pub struct RequestTimer;

#[rocket::async_trait]
impl Fairing for RequestTimer {
    fn info(&self) -> Info {
        Info {
            name: "Request Timer",
            kind: Kind::Request | Kind::Response
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        request.local_cache(|| Instant::now());
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let start_time = request.local_cache(|| Instant::now());
        let duration = start_time.elapsed();
        
        println!("Path: {} - Execution Time: {:.2?}", request.uri(), duration);
        
        // Add timing header to response
        response.set_header(rocket::http::Header::new(
            "X-Response-Time",
            format!("{:.2?}", duration)
        ));
    }
}