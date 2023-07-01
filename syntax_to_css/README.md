# Sublime Text `.tmTheme` to CSS File Converter

This is a very simple utility that exposes [syntect](https://github.com/trishume/syntect)'s built in capability to load
and parse Sublime Text `.tmTheme`-format themes and emit equivalent CSS themes for use with syntect's syntax
highlighting.

This utility was put together so that it is a simple matter of running a quick command via the CLI to do this
rather than having to write some small bit of code yourself to do the same thing via syntect.

## Usage

```text
syntax_to_css /path/to/your-theme.tmTheme
```

This will result in a CSS file being written to `/path/to/your-theme.css`.

## Obtaining Themes

You can of course use anyone's custom Sublime Text themes with this tool, provided they are distributed in `.tmTheme`
format and **not** as `.sublime-theme` files.

Alternatively, if you just want to generate CSS files from Sublime Text's default built-in themes, you can grab these
from your Sublime Text install. The location of these files will vary, but for example if on a Linux system you
installed Sublime Text to `/opt/sublime_text`, you can find a file at `/opt/sublime_text/Packages/Color Scheme - Legacy.sublime-package`
which is actually just a ZIP archive with a different file extension. Inside this archive you can find a number of
different `.tmTheme`-format themes that you can use with this tool to generate CSS files from.
