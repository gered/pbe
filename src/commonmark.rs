use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Debug, thiserror::Error)]
pub enum CommonMarkError {
	#[error("Syntax highlighting error")]
	SyntectError(#[from] syntect::Error),
}

struct SyntectContext {
	syntax_set: SyntaxSet,
}

pub struct CommonMarkRenderer {
	syntect_context: SyntectContext,
}

impl CommonMarkRenderer {
	pub fn new() -> Self {
		let syntax_set = SyntaxSet::load_defaults_newlines();
		CommonMarkRenderer { syntect_context: SyntectContext { syntax_set } }
	}

	fn highlight_code(&self, code: &str, language: &str) -> Result<String, CommonMarkError> {
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
		Ok(format!("<pre><code>{}</code></pre>", html_generator.finalize()))
	}

	fn highlight_codeblocks<'input>(
		&self,
		events: Parser<'input, '_>,
	) -> Result<impl Iterator<Item = Event<'input>> + 'input, CommonMarkError> {
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

	pub fn render_to_html(&self, s: &str) -> Result<String, CommonMarkError> {
		let parser = Parser::new_ext(s, pulldown_cmark::Options::all());
		let events = self.highlight_codeblocks(parser)?;
		let mut output = String::new();
		pulldown_cmark::html::push_html(&mut output, events);
		Ok(output)
	}
}
