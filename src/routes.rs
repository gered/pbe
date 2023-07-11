use actix_web::web::Redirect;
use actix_web::{web, Either, HttpRequest, HttpResponse, Responder};

use crate::site;

fn not_found() -> HttpResponse {
	HttpResponse::NotFound().body("not found")
}

#[actix_web::route("/", method = "GET", method = "HEAD")]
pub async fn latest_posts(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET / -> latest_posts()");
	data.serve_latest_post()
}

#[actix_web::route("/tag/{tag}", method = "GET", method = "HEAD")]
pub async fn latest_posts_by_tag(path: web::Path<(String,)>, data: web::Data<site::SiteService>) -> impl Responder {
	let tag = path.into_inner().0;
	log::debug!("GET /tag/{0} -> latest_posts_by_tag(), tag = {0}", tag);
	data.serve_posts_by_tag(&tag)
}

#[actix_web::route("/archive", method = "GET", method = "HEAD")]
pub async fn posts_archive(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET /archive -> posts_archive()");
	data.serve_posts_archive()
}

#[actix_web::route("/rss", method = "GET", method = "HEAD")]
pub async fn rss_feed(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET /rss -> rss_feed()");
	data.serve_rss_feed()
}

pub async fn site_content(
	req: HttpRequest,
	data: web::Data<site::SiteService>,
) -> Result<Either<HttpResponse, Redirect>, site::SiteError> {
	log::debug!("GET {} -> fallback to site_content()", req.path());
	if let Some(response) = data.serve_content_by_url(&req)? {
		Ok(response)
	} else {
		Ok(Either::Left(not_found()))
	}
}
