use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Redirect;
use actix_web::{Either, HttpRequest, HttpResponse};
use chrono::{Datelike, TimeZone};
use itertools::Itertools;

use crate::{config, markdown};

type UriPath = String;
type Tag = String;

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
	#[error("Content rendering I/O error with path {0}")]
	IOError(PathBuf, #[source] std::io::Error),

	#[error("Markdown error")]
	MarkdownError(#[from] markdown::MarkdownError),

	#[error("Markdown rendering error with path {0}")]
	MarkdownRenderingError(PathBuf, #[source] markdown::MarkdownError),
}

pub struct ContentRenderer {
	markdown_renderer: markdown::MarkdownRenderer,
}

impl ContentRenderer {
	pub fn new(server_config: &config::Server) -> Result<Self, ContentError> {
		Ok(ContentRenderer {
			//
			markdown_renderer: markdown::MarkdownRenderer::new(server_config)?,
		})
	}

	pub fn render(&self, path: &PathBuf) -> Result<String, ContentError> {
		let raw_content = match std::fs::read_to_string(path) {
			Err(e) => return Err(ContentError::IOError(path.clone(), e)),
			Ok(s) => s,
		};
		match path.extension().unwrap_or_default().to_str() {
			Some("md") => match self.markdown_renderer.render_to_html(&raw_content) {
				Err(e) => return Err(ContentError::MarkdownRenderingError(path.clone(), e)),
				Ok(output) => Ok(output),
			},
			Some("html") | Some("htm") => Ok(raw_content),
			_ => Ok(raw_content),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SiteError {
	#[error("Content rendering error")]
	ContentError(#[from] ContentError),

	#[error("Tera templates error")]
	TeraError(#[from] tera::Error),
}

impl actix_web::error::ResponseError for SiteError {
	fn error_response(&self) -> HttpResponse<BoxBody> {
		log::error!("Error response: {:?}", self);
		let status_code = self.status_code();
		HttpResponse::build(status_code) //
			.content_type(ContentType::plaintext())
			.body(format!("{status_code}\n\n{:#?}", self))
	}
}

pub struct AlternateUrlMappings {
	mapping: HashMap<UriPath, UriPath>,
}

impl AlternateUrlMappings {
	pub fn new() -> Self {
		AlternateUrlMappings { mapping: HashMap::new() }
	}

	#[inline]
	pub fn get(&self, alternate_url: &UriPath) -> Option<&UriPath> {
		self.mapping.get(alternate_url)
	}

	#[inline]
	pub fn add_mapping(&mut self, alternate_url: &UriPath, current_url: &UriPath) {
		self.mapping.insert(alternate_url.clone(), current_url.clone());
	}

	pub fn add_mappings(&mut self, alternate_urls: &[UriPath], current_url: &UriPath) {
		for url in alternate_urls.iter() {
			self.add_mapping(url, current_url);
		}
	}
}

pub struct PostsByTag {
	mapping: HashMap<Tag, Vec<usize>>,
}

impl PostsByTag {
	pub fn new() -> Self {
		PostsByTag { mapping: HashMap::new() }
	}

	#[inline]
	pub fn get(&self, tag: &Tag) -> Option<&[usize]> {
		self.mapping.get(tag).map(|x| x.as_slice())
	}

	pub fn add_mapping(&mut self, post_index: usize, tag: &Tag) {
		if let Some(indices) = self.mapping.get_mut(tag) {
			indices.push(post_index);
		} else {
			self.mapping.insert(tag.clone(), vec![post_index]);
		}
	}

	pub fn add_mappings(&mut self, post_index: usize, tags: &[Tag]) {
		for tag in tags.iter() {
			self.add_mapping(post_index, tag);
		}
	}
}

#[derive(serde::Serialize)]
pub struct Post {
	pub url: UriPath,
	pub title: String,
	pub content_html: String,
	#[serde(serialize_with = "crate::util::serialize_naivedatetime_to_i64")]
	pub date: chrono::NaiveDateTime,
	pub tags: Vec<Tag>,
}

impl Post {
	pub fn try_from(value: config::Post, content_renderer: &ContentRenderer) -> Result<Self, SiteError> {
		let url = format!(
			"/{:04}/{:02}/{:02}/{}", //
			value.date.year(),
			value.date.month(),
			value.date.day(),
			value.slug
		);
		let content_html = content_renderer.render(&value.file_path)?;
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

impl Page {
	pub fn try_from(value: config::Page, content_renderer: &ContentRenderer) -> Result<Self, SiteError> {
		let content_html = content_renderer.render(&value.file_path)?;
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
	pub template_renderer: tera::Tera,
	pub pages: Vec<Page>,
	pub posts: Vec<Post>,
	pub pages_by_url: HashMap<UriPath, usize>,
	pub posts_by_url: HashMap<UriPath, usize>,
	pub alternate_url_mappings: AlternateUrlMappings,
	pub post_tag_mappings: PostsByTag,
	pub rss: RssMetadata,
}

impl SiteContent {
	pub fn new(
		pages_config: config::Pages,
		posts_config: config::Posts,
		server_config: &config::Server,
		content_renderer: &ContentRenderer,
	) -> Result<Self, SiteError> {
		let mut templates_path = PathBuf::from(&server_config.templates_path);
		templates_path.push("**/*");
		log::debug!("Using templates path: {:?}", templates_path);
		let template_renderer = tera::Tera::new(templates_path.as_path().to_str().unwrap())?;
		log::debug!(
			"Templates loaded and parsed from the templates path: {:?}",
			template_renderer.get_template_names().collect::<Vec<&str>>()
		);

		let mut alternate_url_mappings = AlternateUrlMappings::new();
		let mut post_tag_mappings = PostsByTag::new();

		// load pages
		let mut pages = Vec::new();
		let mut pages_by_url = HashMap::new();
		for (index, page_config) in pages_config.pages.iter().enumerate() {
			let page = Page::try_from(page_config.clone(), content_renderer)?;

			if let Some(old_urls) = &page_config.alternate_urls {
				alternate_url_mappings.add_mappings(old_urls, &page.url);
			}

			pages_by_url.insert(page.url.clone(), index);
			pages.push(page);
		}

		// load posts, iterating over the config's list of posts in descending order by date so that
		// our final post list is pre-sorted this way, as well as the post lists per tag
		let mut posts = Vec::new();
		let mut posts_by_url = HashMap::new();
		for (index, post_config) in posts_config.posts.iter().sorted_by(|a, b| b.date.cmp(&a.date)).enumerate() {
			let post = Post::try_from(post_config.clone(), content_renderer)?;

			if let Some(old_urls) = &post_config.alternate_urls {
				alternate_url_mappings.add_mappings(old_urls, &post.url);
			}

			posts_by_url.insert(post.url.clone(), index);
			post_tag_mappings.add_mappings(index, &post.tags);
			posts.push(post);
		}

		let rss = RssMetadata::from(posts_config.rss);

		Ok(SiteContent {
			template_renderer,
			pages,
			posts,
			pages_by_url,
			posts_by_url,
			alternate_url_mappings,
			post_tag_mappings,
			rss,
		})
	}

	pub fn get_page_by_url(&self, url: &UriPath) -> Option<&Page> {
		self.pages_by_url.get(url).map(|index| self.pages.get(*index).unwrap())
	}

	pub fn get_post_by_url(&self, url: &UriPath) -> Option<&Post> {
		self.posts_by_url.get(url).map(|index| self.posts.get(*index).unwrap())
	}

	pub fn get_content_at(&self, url: &UriPath) -> Option<Content> {
		if let Some(new_url) = self.alternate_url_mappings.get(url) {
			Some(Content::Redirect(new_url.clone()))
		} else if let Some(post) = self.get_post_by_url(url) {
			Some(Content::Post(post))
		} else if let Some(page) = self.get_page_by_url(url) {
			Some(Content::Page(page))
		} else {
			None
		}
	}

	pub fn get_posts_ordered_by_date(&self) -> &[Post] {
		self.posts.as_slice()
	}

	pub fn get_posts_with_tag_ordered_by_date(&self, tag: &Tag) -> Vec<&Post> {
		let mut posts = Vec::new();
		if let Some(post_indices) = self.post_tag_mappings.get(tag) {
			for post_index in post_indices.iter() {
				posts.push(self.posts.get(*post_index).unwrap())
			}
		}
		posts
	}

	pub fn get_latest_post(&self) -> Option<&Post> {
		self.posts.first()
	}
}

pub struct RefreshWrapper<T> {
	pub data: T,
}

impl<T> RefreshWrapper<T> {
	pub fn new(data: T) -> Self {
		RefreshWrapper { data }
	}
}

impl<T> Deref for RefreshWrapper<T> {
	type Target = T;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

pub struct SiteService {
	pub server_config: config::Server,
	pub content_renderer: ContentRenderer,
	pub content: RwLock<RefreshWrapper<SiteContent>>,
}

impl SiteService {
	pub fn new(
		server_config: config::Server,
		pages_config: config::Pages,
		posts_config: config::Posts,
	) -> Result<Self, SiteError> {
		let content_renderer = ContentRenderer::new(&server_config)?;
		let content =
			RefreshWrapper::new(SiteContent::new(pages_config, posts_config, &server_config, &content_renderer)?);
		Ok(SiteService {
			server_config, //
			content_renderer,
			content: RwLock::new(content),
		})
	}

	pub fn refresh_content(&self, pages_config: config::Pages, posts_config: config::Posts) -> Result<(), SiteError> {
		let mut existing_content = self.content.write().expect("SiteContent write lock failed"); // TODO: better error handling
		log::debug!("Obtained write lock on SiteContent instance");
		let content = SiteContent::new(pages_config, posts_config, &self.server_config, &self.content_renderer)?;
		log::debug!("New SiteContent instance built successfully");
		existing_content.data = content;
		Ok(())
	}

	pub fn serve_latest_post(&self) -> Result<HttpResponse, SiteError> {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let post = content.get_latest_post();
		let mut context = tera::Context::new();
		if let Some(post) = post {
			context.insert("post", post);
		}
		let response_body = content.template_renderer.render("latest_post.html", &context)?;
		Ok(HttpResponse::Ok().content_type(ContentType::html()).body(response_body))
	}

	pub fn serve_posts_by_tag(&self, tag: &Tag) -> Result<HttpResponse, SiteError> {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let posts = content.get_posts_with_tag_ordered_by_date(tag);
		let mut context = tera::Context::new();
		context.insert("tag", tag);
		context.insert("posts", &posts);
		let response_body = content.template_renderer.render("tag.html", &context)?;
		Ok(HttpResponse::Ok().content_type(ContentType::html()).body(response_body))
	}

	pub fn serve_posts_archive(&self) -> Result<HttpResponse, SiteError> {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let posts = content.get_posts_ordered_by_date();
		let mut context = tera::Context::new();
		context.insert("posts", &posts);
		let response_body = content.template_renderer.render("archive.html", &context)?;
		Ok(HttpResponse::Ok().content_type(ContentType::html()).body(response_body))
	}

	pub fn serve_rss_feed(&self) -> Result<HttpResponse, SiteError> {
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
						.pub_date(chrono::Local.from_local_datetime(&post.date).unwrap().to_string())
						.build()
				})
				.collect::<Vec<rss::Item>>(),
		);
		let response_body = channel.to_string();
		Ok(HttpResponse::Ok().content_type("application/rss+xml").body(response_body))
	}

	pub fn serve_content_by_url(&self, req: &HttpRequest) -> Result<Option<Either<HttpResponse, Redirect>>, SiteError> {
		let content = self.content.read().expect("SiteContent read lock failed"); // TODO: better error handling
		let url = String::from(req.path());
		match content.get_content_at(&url) {
			Some(Content::Page(page)) => {
				log::debug!("Found page content at {}", req.path());
				let mut context = tera::Context::new();
				context.insert("page", page);
				let rendered = content.template_renderer.render("page.html", &context)?;
				Ok(Some(Either::Left(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))))
			}
			Some(Content::Post(post)) => {
				log::debug!("Found post content at {}", req.path());
				let mut context = tera::Context::new();
				context.insert("post", post);
				let rendered = content.template_renderer.render("post.html", &context)?;
				Ok(Some(Either::Left(HttpResponse::Ok().content_type(ContentType::html()).body(rendered))))
			}
			Some(Content::Redirect(url)) => {
				log::debug!("Found redirect at {}", req.path());
				Ok(Some(Either::Right(Redirect::to(url).using_status_code(StatusCode::MOVED_PERMANENTLY))))
			}
			None => {
				log::debug!("No matching content at {}", req.path());
				Ok(None)
			}
		}
	}
}
