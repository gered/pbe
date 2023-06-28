use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Server {
	pub bind_addr: String,
	pub bind_port: u16,
	pub static_files_path: PathBuf,
	pub templates_path: PathBuf,
	pub pages_path: PathBuf,
	pub posts_path: PathBuf,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Rss {
	pub title: String,
	pub description: String,
	pub url: String,
	pub count: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Page {
	pub file_path: PathBuf,
	pub title: String,
	pub url: String,
	pub alternate_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Pages {
	pub pages: Vec<Page>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Post {
	pub file_path: PathBuf,
	pub title: String,
	#[serde(deserialize_with = "crate::util::deserialize_string_to_naivedatetime")]
	pub date: chrono::NaiveDateTime,
	pub slug: String,
	pub alternate_urls: Option<Vec<String>>,
	pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Posts {
	pub posts: Vec<Post>,
	pub rss: Rss,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("Json config I/O error with path {0}")]
	IOError(PathBuf, #[source] std::io::Error),

	#[error("Json deserialization error: {0}")]
	SerdeJsonError(String),
}

fn load_json_config<T>(path: &PathBuf) -> Result<T, ConfigError>
where
	T: serde::de::DeserializeOwned,
{
	let file = File::open(path).map_err(|e| ConfigError::IOError(path.clone(), e))?;
	let mut reader = BufReader::new(file);
	match serde_json::from_reader(&mut reader) {
		Ok(deserialized) => Ok(deserialized),
		Err(err) => Err(ConfigError::SerdeJsonError(err.to_string())),
	}
}

pub fn load_server(path: &PathBuf, site_root: &PathBuf) -> Result<Server, ConfigError> {
	log::info!("Loading server json config from {:?}", path);
	let mut server_config: Server = load_json_config(path)?;
	server_config.static_files_path = [site_root, &server_config.static_files_path].iter().collect();
	server_config.templates_path = [site_root, &server_config.templates_path].iter().collect();
	server_config.pages_path = [site_root, &server_config.pages_path].iter().collect();
	server_config.posts_path = [site_root, &server_config.posts_path].iter().collect();
	Ok(server_config)
}

pub fn load_content(
	pages_path: &PathBuf,
	posts_path: &PathBuf,
	server_config: &Server,
) -> Result<(Pages, Posts), ConfigError> {
	log::info!("Loading pages json config from {:?}", pages_path);
	let mut pages: Pages = load_json_config(pages_path)?;
	for page in pages.pages.iter_mut() {
		page.file_path = [&server_config.pages_path, &page.file_path].iter().collect();
	}
	log::info!("Loading posts json config from {:?}", posts_path);
	let mut posts: Posts = load_json_config(posts_path)?;
	for post in posts.posts.iter_mut() {
		post.file_path = [&server_config.posts_path, &post.file_path].iter().collect();
	}
	Ok((pages, posts))
}
