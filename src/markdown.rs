use itertools::Itertools;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::config;

#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
	#[error("Syntax highlighting error")]
	SyntectError(#[from] syntect::Error),

	#[error("Syntax loading error")]
	SyntectLoadingError(#[from] syntect::LoadingError),
}

struct SyntectContext {
	syntax_set: SyntaxSet,
}

pub struct MarkdownRenderer {
	syntect_context: SyntectContext,
}

impl MarkdownRenderer {
	pub fn new(server_config: &config::Server) -> Result<Self, MarkdownError> {
		let syntax_set = if let Some(syntaxes_path) = &server_config.syntaxes_path {
			log::debug!("Using syntaxes path: {:?}", syntaxes_path);
			let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
			builder.add_from_folder(syntaxes_path, true)?;
			builder.build()
		} else {
			log::debug!("Using default syntaxes only.");
			SyntaxSet::load_defaults_newlines()
		};
		let syntax_names: Vec<&String> =
			syntax_set.syntaxes().iter().flat_map(|syntax| &syntax.file_extensions).sorted().collect();
		log::debug!("Syntaxes loaded: {:?}", syntax_names);
		Ok(MarkdownRenderer { syntect_context: SyntectContext { syntax_set } })
	}

	fn highlight_code(&self, code: &str, language: &str) -> Result<String, MarkdownError> {
		let syntax = self
			.syntect_context
			.syntax_set
			.find_syntax_by_extension(language)
			.unwrap_or_else(|| self.syntect_context.syntax_set.find_syntax_plain_text());

		let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
			syntax,
			&self.syntect_context.syntax_set,
			ClassStyle::SpacedPrefixed { prefix: "sh-" },
		);
		for line in LinesWithEndings::from(code) {
			html_generator.parse_html_for_line_which_includes_newline(line)?;
		}
		// the "sh-code" css class is what syntect will generate for the top-level code container that includes
		// things like the background color and default text foreground color.
		// the inner classname we're generating with the language included in it is not used for anything. it's
		// just a marker that includes the name of the language syntax used
		Ok(format!(
			"<pre class=\"sh-code\"><code class=\"{}\">{}</code></pre>",
			if !language.is_empty() { format!("syntax-{}", language) } else { String::new() },
			html_generator.finalize()
		))
	}

	fn highlight_codeblocks<'input>(
		&self,
		events: Parser<'input, '_>,
	) -> Result<impl Iterator<Item = Event<'input>> + 'input, MarkdownError> {
		let mut modified_events = Vec::new();
		let mut code_buffer = String::new();
		let mut is_in_code_block = false;

		for event in events {
			match event {
				Event::Start(Tag::CodeBlock(_)) => {
					is_in_code_block = true;
					code_buffer.clear();
				}
				Event::End(Tag::CodeBlock(kind)) => {
					if is_in_code_block {
						let language = if let CodeBlockKind::Fenced(language) = kind {
							language.to_string()
						} else {
							String::new()
						};
						let html = self.highlight_code(&code_buffer, &language)?;
						modified_events.push(Event::Html(CowStr::Boxed(html.into())));
						is_in_code_block = false;
					}
				}
				Event::Text(text) => {
					if is_in_code_block {
						code_buffer.push_str(&text);
					} else {
						modified_events.push(Event::Text(text))
					}
				}
				event => modified_events.push(event),
			}
		}

		Ok(modified_events.into_iter())
	}

	pub fn render_to_html(&self, s: &str) -> Result<String, MarkdownError> {
		let parser = Parser::new_ext(s, pulldown_cmark::Options::all());
		let events = self.highlight_codeblocks(parser)?;
		let mut output = String::new();
		pulldown_cmark::html::push_html(&mut output, events);
		Ok(output)
	}
}
