use std::env;
use std::path::{Path, PathBuf};

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use anyhow::Context;

mod config;
mod markdown;
mod routes;
mod site;
mod util;
mod watcher;

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
	println!(
		"Build version {0}, git hash {1}, built at {2}",
		env!("CARGO_PKG_VERSION"),
		env!("GIT_HASH"),
		env!("BUILD_TS")
	);

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

		// note that we do not want to watch the static files path. there's no need, as nothing in there that is
		// being cached by us here
		let watch_paths = vec![
			pages_config_path.clone(),
			posts_config_path.clone(),
			server_config.pages_path.clone(),
			server_config.posts_path.clone(),
			server_config.templates_path.clone(),
		];
		let watcher_handle = spawn_watcher(watch_paths, pages_config_path, posts_config_path, data.clone());

		log::info!(
			"Spawning HTTP server for site, listening on {}:{} ...",
			server_config.bind_addr,
			server_config.bind_port
		);

		HttpServer::new(move || {
			App::new() //
				.app_data(data.clone())
				.wrap(actix_web::middleware::NormalizePath::trim())
				.service(routes::latest_posts)
				.service(routes::latest_posts_by_tag)
				.service(routes::posts_archive)
				.service(routes::rss_feed)
				.service(Files::new("/", &server_config.static_files_path))
				.default_service(web::get().to(routes::site_content))
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
