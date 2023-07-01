use std::env;
use std::path::{Path, PathBuf};

use actix_files::Files;
use actix_web::web::Redirect;
use actix_web::{web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Context;

mod config;
mod markdown;
mod site;
mod util;
mod watcher;

fn not_found() -> HttpResponse {
	HttpResponse::NotFound().body("not found")
}

#[actix_web::get("/")]
async fn latest_posts(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET / -> latest_posts()");
	data.serve_latest_post()
}

#[actix_web::get("/tag/{tag}/")]
async fn latest_posts_by_tag(path: web::Path<(String,)>, data: web::Data<site::SiteService>) -> impl Responder {
	let tag = path.into_inner().0;
	log::debug!("GET /tag/{0}/ -> latest_posts_by_tag(), tag = {0}", tag);
	data.serve_posts_by_tag(&tag)
}

#[actix_web::get("/archive/")]
async fn posts_archive(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET /archive/ -> posts_archive()");
	data.serve_posts_archive()
}

#[actix_web::get("/rss/")]
async fn rss_feed(data: web::Data<site::SiteService>) -> impl Responder {
	log::debug!("GET /rss/ -> rss_feed()");
	data.serve_rss_feed()
}

async fn site_content(req: HttpRequest, data: web::Data<site::SiteService>) -> Either<HttpResponse, Redirect> {
	log::debug!("GET {} -> fallback to site_content()", req.path());
	if let Some(response) = data.serve_content_by_url(&req) {
		response
	} else {
		Either::Left(not_found())
	}
}

fn spawn_watcher(
	watch_paths: Vec<PathBuf>,
	pages_config_path: PathBuf,
	posts_config_path: PathBuf,
	data: web::Data<site::SiteService>,
) -> tokio::task::JoinHandle<()> {
	log::info!("Spawning filesystem watcher for paths {:?}", watch_paths);
	tokio::spawn(async move {
		watcher::debounce_watch(&watch_paths, move |event| {
			match event {
				Ok(_) => {
					// right now we don't actually care which file was modified. just always rebuild the whole thing.
					// this is also why using a debounced watch is important and probably should use a somewhat long
					// debounce time in practice, just in case someone is doing a lengthy file upload to a remote
					// server or something of that nature which takes more than 1-2 seconds.
					// TODO: maybe try to selectively rebuild only what was changed? meh.
					log::warn!(
						"Modification to file(s) in watched paths detected, beginning re-generation of SiteContent"
					);

					log::info!("Reloading content configs");
					let (pages_config, posts_config) =
						match config::load_content(&pages_config_path, &posts_config_path, &data.server_config) {
							Ok(configs) => configs,
							Err(err) => {
								log::error!("Error reloading content configs: {:?}", err);
								return;
							}
						};

					log::info!("Re-generating SiteContent");
					if let Err(err) = data.refresh_content(pages_config, posts_config) {
						log::error!("Error re-generating SiteContent: {:?}", err);
						return;
					}

					log::info!("Finished re-generating SiteContent");
				}
				Err(errors) => {
					for error in errors {
						log::error!("debounce_watch event handler error: {:?}", error);
					}
				}
			}
		})
		.await
		.unwrap()
	})
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
	let log_level = env::var("LOG_LEVEL").map_or(String::from("info"), |value| value.to_lowercase());
	simple_log::new(
		simple_log::LogConfigBuilder::builder() //
			.level(log_level)
			.output_console()
			.build(),
	)
	.map_err(|err| anyhow::anyhow!(err))?;

	println!("PBE - Personal Blog Engine - https://github.com/gered/pbe");

	// manually handling args because
	// 1) i have very simple needs
	// 2) the common crate options are kinda complex and/or have poorly laid out docs (especially so when it comes to
	//    the typical simple use-case "getting started" stuff ... bleh)
	// so ... whatever!

	let mut args: Vec<String> = env::args().collect();
	args.remove(0); // normally the path of the executable itself. TODO: when is this not true? probably only some exotic environments i don't give a shit about ...?

	let first_arg = args.first().unwrap_or(&String::new()).to_lowercase();
	if first_arg == "-h" || first_arg == "--help" {
		println!("Usage: pbe <SITE_ROOT>");
		println!("Where SITE_ROOT is a path that contains the config files and all content and web resources.");
		Ok(())
	} else {
		let site_root = if first_arg.is_empty() {
			env::current_dir()? //
		} else {
			Path::new(&first_arg).canonicalize()?
		};
		log::info!("Using site root {:?}", site_root);

		let server_config_path: PathBuf = [&site_root, &"server.yml".into()].iter().collect();
		let pages_config_path: PathBuf = [&site_root, &"pages.yml".into()].iter().collect();
		let posts_config_path: PathBuf = [&site_root, &"posts.yml".into()].iter().collect();

		log::info!("Loading config ...");
		let server_config = config::load_server(&server_config_path, &site_root) //
			.context("Loading server config")?;
		let (pages_config, posts_config) = config::load_content(&pages_config_path, &posts_config_path, &server_config) //
			.context("Loading content configs")?;

		log::info!("Initializing site data and content ...");
		let site_service = site::SiteService::new(server_config.clone(), pages_config, posts_config)
			.context("Constructing SiteService instance")?;
		let data = web::Data::new(site_service);

		let watch_paths = vec![pages_config_path.clone(), posts_config_path.clone()];
		let watcher_handle = spawn_watcher(watch_paths, pages_config_path, posts_config_path, data.clone());

		log::info!(
			"Spawning HTTP server for site, listening on {}:{} ...",
			server_config.bind_addr,
			server_config.bind_port
		);

		HttpServer::new(move || {
			App::new() //
				.app_data(data.clone())
				.service(latest_posts)
				.service(latest_posts_by_tag)
				.service(posts_archive)
				.service(rss_feed)
				.service(Files::new("/", &server_config.static_files_path))
				.default_service(web::get().to(site_content))
		})
		.bind((server_config.bind_addr.clone(), server_config.bind_port))
		.with_context(|| format!("Binding HTTP server on {}:{}", server_config.bind_addr, server_config.bind_port))?
		.run()
		.await
		.map_err(anyhow::Error::from)?;

		log::info!("Aborting filesystem watcher");
		watcher_handle.abort();

		log::info!("Finished!");
		Ok(())
	}
}
