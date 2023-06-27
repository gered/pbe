use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use actix_web::web::Redirect;
use actix_web::{Either, HttpRequest, HttpResponse};
use chrono::{Datelike, TimeZone};
use itertools::Itertools;

use crate::{config, util};

type UriPath = String;
type Tag = String;

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
	#[error("Content rendering I/O error with path {0}")]
	IOError(PathBuf, #[source] std::io::Error),
}

fn render_content(path: &PathBuf) -> Result<String, ContentError> {
	let raw_content = match std::fs::read_to_string(path) {
		Err(e) => return Err(ContentError::IOError(path.clone(), e)),
		Ok(s) => s,
	};
	match path.extension().unwrap_or_default().to_str() {
		Some("md") => {
			let parser = pulldown_cmark::Parser::new_ext(&raw_content, pulldown_cmark::Options::all());
			let mut output = String::new();
			// TODO: use write_html() instead because that can actually return errors instead of just panicking
			pulldown_cmark::html::push_html(&mut output, parser);
			Ok(output)
		}
		Some("html") | Some("htm") => Ok(raw_content),
		_ => Ok(raw_content),
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SiteError {
	#[error("Content rendering error")]
	ContentError(#[from] ContentError),

	#[error("Tera templates error")]
	TeraError(#[from] tera::Error),
}

pub struct OldUrlMappings {
	mapping: HashMap<UriPath, UriPath>,
}

impl OldUrlMappings {
	pub fn new() -> Self {
		OldUrlMappings { mapping: HashMap::new() }
	}

	#[inline]
	pub fn get(&self, old_url: &UriPath) -> Option<&UriPath> {
		self.mapping.get(old_url)
	}

	#[inline]
	pub fn add_mapping(&mut self, old_url: &UriPath, new_url: &UriPath) {
		self.mapping.insert(old_url.clone(), new_url.clone());
	}

	pub fn add_mappings(&mut self, old_urls: &[UriPath], new_url: &UriPath) {
		for old_url in old_urls.iter() {
			self.add_mapping(old_url, new_url);
		}
	}
}

pub struct UrlsByTag {
	mapping: HashMap<Tag, Vec<UriPath>>,
}

impl UrlsByTag {
	pub fn new() -> Self {
		UrlsByTag { mapping: HashMap::new() }
	}

	#[inline]
	pub fn get(&self, tag: &Tag) -> Option<&[UriPath]> {
		self.mapping.get(tag).map(|x| x.as_slice())
	}

	pub fn add_mapping(&mut self, url: &UriPath, tag: &Tag) {
		if let Some(urls) = self.mapping.get_mut(tag) {
			urls.push(url.clone());
		} else {
			let urls = vec![url.clone()];
			self.mapping.insert(tag.clone(), urls);
		}
	}

	pub fn add_mappings(&mut self, url: &UriPath, tags: &[Tag]) {
		for tag in tags.iter() {
			self.add_mapping(url, tag);
		}
	}
}

#[derive(serde::Serialize)]
pub struct Post {
	pub url: UriPath,
	pub title: String,
	pub content_html: String,
	#[serde(serialize_with = "crate::util::serialize_naivedate")]
	pub date: chrono::NaiveDate,
	pub tags: Vec<Tag>,
}

impl TryFrom<config::Post> for Post {
	type Error = SiteError;

	fn try_from(value: config::Post) -> Result<Self, Self::Error> {
		let url = format!("/{:04}/{:02}/{:02}/{}", value.date.year(), value.date.month(), value.date.day(), value.slug);
		let content_html = render_content(&value.file_path)?;
		let tags = value.tags.map_or_else(|| Vec::new(), |x| x.clone());
		Ok(Post {
			url, //
			title: value.title,
			content_html,
			date: value.date,
			tags,
		})
	}
}

#[derive(serde::Serialize)]
pub struct Page {
	pub url: UriPath,
	pub title: String,
	pub content_html: String,
}

impl TryFrom<config::Page> for Page {
	type Error = SiteError;

	fn try_from(value: config::Page) -> Result<Self, Self::Error> {
		let content_html = render_content(&value.file_path)?;
		Ok(Page {
			url: value.url, //
			title: value.title,
			content_html,
		})
	}
}

pub struct RssMetadata {
	pub title: String,
	pub description: String,
	pub url: String,
	pub count: usize,
}

impl From<config::Rss> for RssMetadata {
	fn from(value: config::Rss) -> Self {
		RssMetadata {
			title: value.title, //
			description: value.description,
			url: value.url,
			count: value.count,
		}
	}
}

pub enum Content<'a> {
	Page(&'a Page),
	Post(&'a Post),
	Redirect(UriPath),
}

pub struct SiteContent {
	pub pages: HashMap<UriPath, Page>,
	pub posts: HashMap<UriPath, Post>,
	pub old_url_mappings: OldUrlMappings,
	pub post_tag_mappings: UrlsByTag,
	pub sorted_post_urls: Vec<UriPath>,
	pub rss: RssMetadata,
}

impl SiteContent {
	pub fn new(pages_config: config::Pages, posts_config: config::Posts) -> Result<Self, SiteError> {
		let mut old_url_mappings = OldUrlMappings::new();
		let mut post_tag_mappings = UrlsByTag::new();
		let mut sorted_post_urls = Vec::<UriPath>::new();

		// load pages
		let mut pages = HashMap::<UriPath, Page>::new();
		for page_config in pages_config.pages.iter() {
			let page = Page::try_from(page_config.clone())?;

			if let Some(old_urls) = &page_config.old_urls {
				old_url_mappings.add_mappings(old_urls, &page.url);
			}

			pages.insert(page.url.clone(), page);
		}

		// load posts
		let mut posts = HashMap::<UriPath, Post>::new();
		for post_config in posts_config.posts.iter() {
			let post = Post::try_from(post_config.clone())?;

			if let Some(old_urls) = &post_config.old_urls {
				old_url_mappings.add_mappings(old_urls, &post.url);
			}

			posts.insert(post.url.clone(), post);
		}

		// build pre-sorted post urls table. as well, build the post url by tag mapping here so that
		// the post urls for each tag will already be ordered by date
		for post in posts.values().sorted_by(|a, b| b.date.cmp(&a.date)) {
			sorted_post_urls.push(post.url.clone());
			post_tag_mappings.add_mappings(&post.url, &post.tags);
		}

		let rss = RssMetadata::from(posts_config.rss);

		Ok(SiteContent { pages, posts, old_url_mappings, post_tag_mappings, sorted_post_urls, rss })
	}

	pub fn get_content_at(&self, url: &UriPath) -> Option<Content> {
		if let Some(new_url) = self.old_url_mappings.get(url) {
			Some(Content::Redirect(new_url.clone()))
		} else if let Some(post) = self.posts.get(url) {
			Some(Content::Post(post))
		} else if let Some(page) = self.pages.get(url) {
			Some(Content::Page(page))
		} else {
			None
		}
	}

	pub fn get_posts_ordered_by_date(&self) -> Vec<&Post> {
		self.sorted_post_urls.iter().map(|post_url| self.posts.get(post_url).unwrap()).collect()
	}

	pub fn get_posts_with_tag_ordered_by_date(&self, tag: &Tag) -> Vec<&Post> {
		let mut posts = Vec::new();
		if let Some(post_urls) = self.post_tag_mappings.get(tag) {
			for url in post_urls.iter() {
				posts.push(self.posts.get(url).unwrap())
			}
		}
		posts
	}

	pub fn get_latest_post(&self) -> Option<&Post> {
		self.sorted_post_urls.first().map(|post_url| self.posts.get(post_url).unwrap())
	}
}

pub struct SiteService {
	pub server_config: config::Server,
	pub renderer: tera::Tera,
	pub content: RwLock<SiteContent>,
}

impl SiteService {
	pub fn new(
		server_config: config::Server,
		pages_config: config::Pages,
		posts_config: config::Posts,
	) -> Result<Self, SiteError> {
		let content = SiteContent::new(pages_config, posts_config)?;
		let mut templates_path = PathBuf::from(&server_config.templates_path);
		templates_path.push("**/*");
		log::debug!("Using templates path: {:?}", templates_path);
		let renderer = tera::Tera::new(templates_path.as_path().to_str().unwrap())?;
		log::debug!(
			"Templates loaded and parsed from the templates path: {:?}",
			renderer.get_template_names().collect::<Vec<&str>>()
		);
		Ok(SiteService {
			server_config, //
			renderer,
			content: RwLock::new(content),
		})
	}

	pub fn serve_latest_post(&self) -> HttpResponse {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let post = content.get_latest_post();
		let mut context = tera::Context::new();
		if let Some(post) = post {
			context.insert("post", post);
		}
		HttpResponse::Ok().body(self.renderer.render("post.html", &context).unwrap())
	}

	pub fn serve_posts_by_tag(&self, tag: &Tag) -> HttpResponse {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let posts = content.get_posts_with_tag_ordered_by_date(tag);
		let mut context = tera::Context::new();
		context.insert("tag", tag);
		context.insert("posts", &posts);
		HttpResponse::Ok().body(self.renderer.render("tag.html", &context).unwrap())
	}

	pub fn serve_posts_archive(&self) -> HttpResponse {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let posts = content.get_posts_ordered_by_date();
		let mut context = tera::Context::new();
		context.insert("posts", &posts);
		HttpResponse::Ok().body(self.renderer.render("archive.html", &context).unwrap())
	}

	pub fn serve_rss_feed(&self) -> HttpResponse {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let base_url = url::Url::parse(&content.rss.url).unwrap();
		let posts = content.get_posts_ordered_by_date();
		let mut channel = rss::ChannelBuilder::default() //
			.title(&content.rss.title)
			.description(&content.rss.description)
			.link(&content.rss.url)
			.build();
		channel.set_items(
			posts
				.iter()
				.take(content.rss.count)
				.map(|post| {
					rss::ItemBuilder::default() //
						.title(post.title.clone())
						.content(post.content_html.clone())
						.link(base_url.clone().join(&post.url).unwrap().to_string())
						.pub_date(chrono::Local.from_local_date(&post.date).unwrap().to_string())
						.build()
				})
				.collect::<Vec<rss::Item>>(),
		);
		HttpResponse::Ok().content_type("application/rss+xml").body(channel.to_string())
	}

	pub fn serve_content_by_url(&self, req: &HttpRequest) -> Option<Either<HttpResponse, Redirect>> {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let url = String::from(req.path());
		match content.get_content_at(&url) {
			Some(Content::Page(page)) => {
				log::debug!("Found page content at {}", req.path());
				let mut context = tera::Context::new();
				context.insert("page", page);
				let rendered = self.renderer.render("page.html", &context).unwrap();
				Some(Either::Left(HttpResponse::Ok().body(rendered)))
			}
			Some(Content::Post(post)) => {
				log::debug!("Found post content at {}", req.path());
				let mut context = tera::Context::new();
				context.insert("post", post);
				let rendered = self.renderer.render("post.html", &context).unwrap();
				Some(Either::Left(HttpResponse::Ok().body(rendered)))
			}
			Some(Content::Redirect(url)) => {
				log::debug!("Found redirect at {}", req.path());
				Some(Either::Right(Redirect::to(url).permanent()))
			}
			None => {
				log::debug!("No matching content at {}", req.path());
				None
			}
		}
	}
}
