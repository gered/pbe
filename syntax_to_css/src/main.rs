use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

fn main() -> anyhow::Result<()> {
	println!("Sublime Text Theme to CSS Converter (via Syntect)");

	let mut args: Vec<String> = std::env::args().collect();
	args.remove(0); // normally the path of the executable itself. TODO: when is this not true? probably only some exotic environments i don't give a shit about ...?

	let first_arg = args.first().unwrap_or(&String::new()).clone();
	if first_arg == "-h" || first_arg == "--help" || first_arg.is_empty() {
		println!("Usage: syntax_to_css <FILE>");
		Ok(())
	} else {
		let path = PathBuf::from(first_arg);
		println!("Loading theme {:?}", &path);
		let theme = ThemeSet::get_theme(&path)?;

		println!("Converting to CSS");
		let css = css_for_theme_with_class_style(&theme, ClassStyle::SpacedPrefixed { prefix: "sh-" })?;

		let css_path = path.with_extension("css");
		println!("Writing to file {:?}", &css_path);
		let f = File::create(css_path)?;
		let mut writer = BufWriter::new(f);
		writer.write_all(css.as_bytes())?;

		Ok(())
	}
}
