# PBE: Personal Blog Engine

A service that can serve up a personal website and/or blog, with content being written in HTML, Markdown/CommonMark,
or even just plain text. Mostly this is aimed at the same style of site that nowadays many would prefer to use a
"static site generator" for. I find those things kind of boring personally, preferring to do my own thing instead!

This is my umpteenth take on a personal website / blog over the past ~25 years. This particular iteration is a very
close re-implementation in Rust of the code that I originally wrote in Java a few years ago. The original Java project
was never written in a totally generic way as I tried to do here in this project, but otherwise it was just doing the
exact same things that this project you see here does.

Since this is my first web project done with Rust, I expect there is probably a bunch of "icky" things that I've done
in the code. Be warned!

Also, please note that PBE is a project that is really just intended for my own personal use. Perhaps someone else
may find it useful too. But just keep this in mind as I intend for it to be fairly laser-focused on my own personal
needs and am _not_ interested in expanding it to include a whole variety of different features. 

## Quick Start

This repository contains a full example site. It's not pretty, but it should get you started if you use it as a base
for your own PBE site. Most of the files are probably pretty self-explanatory once you play around with things for a
few minutes, and if you get stuck you can just reference the more detailed documentation found below.

For now, to run the example site, simply do (assuming you've downloaded the PBE binary and cloned this repository
somewhere locally):

```text
pbe /path/to/example-site
```

The argument provided to `pbe` is the **root site path**. If not specified, the current working directory is used.

Once started up successfully, you can access this site in your browser at http://localhost:8080/.

## Overview

PBE is set up to serve up websites that are comprised of:

* A collection of **posts** which have a title, date, zero or more **tags**, and the content written in HTML, Markdown
  or plain text. All posts have a pre-determined format for the URL they are accessed by, based on its title and date.
* Zero or more **pages** which have a title, and content written in HTML, Markdown or plain text. All pages have an
  arbitrary URL that is specified per page in the configuration.
* Static content, served out of a common directory. This includes CSS, images, and any other public web accessible
  content to be served up by any other page or post in the site.

Aside from this, there are some important built-in pages:

* The **homepage** shows just the single most recent post.
* An **archive** page, which shows a list of all posts (only dates, titles, tags), sorted by date in 
  descending order.
* Individual **tag** pages, which show a list very similar to the archive, but only showing a list of posts which
  have that tag.

Finally, there is an RSS feed available (optionally) which includes the most recent posts.

## Running

To run PBE, you will of course need to download the PBE binary.

Then you'll need to assemble a site which will contain a directory structure that looks like the following:

```text
/
    server.yml
    pages.yml
    posts.yml
    
    pages/
        [ one or more .md/.html/etc files containing page content ]
        
    posts/
        [ one or more .md/.html/etc files containing post content ]
    
    static/
        [ your site's publicly accessible web resources, e.g. CSS files, images, etc ]
        
    templates/
        archive.html
        latest_post.html
        page.html
        post.html
        tag.html
```

This directory is your **root site path**. There are a bunch of paths specified in the `server.yml` file which are
assumed to be relative to this root site path.

Once you've assembled your site, you then point PBE at it like so:

```text
pbe /path/to/your/root-site-path
```

At which point your site will be available in your browser at the `bind_addr` and `bind_port` specified in `server.yml`.

## Configuration

### `server.yml`

This is the main configuration file which controls how the website is accessed and where content can be found.

| Key                 | Required? | Description                                                                                                                                                                |
|---------------------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `bind_addr`         | Yes       | The IP address of the network interface to bind the HTTP server on. Usual values would be something like `0.0.0.0` or `127.0.0.1`.                                         |
| `bind_port`         | Yes       | The port to bind the HTTP server on. For example, `8080`.                                                                                                                  |
| `static_files_path` | Yes       | The **relative** path to the directory containing all public web accessible files, e.g. CSS files, images, etc.                                                            |
| `templates_path`    | Yes       | The **relative** path to the directory containing all HTML templates.                                                                                                      |
| `pages_path`        | Yes       | The **relative** path to the directory containing all page Markdown/HTML/text content files.                                                                               |
| `posts_path`        | Yes       | The **relative** path to the directory containing all post Markdown/HTML/text content files.                                                                               |
| `syntaxes_path`     | No        | The **relative** path to the directory containing additional Sublime Text `.sublime-syntax` files to be used for code syntax highlighting when rendering Markdown content. |

Note that all paths are expected to be **relative** and will be evaluated relative to the **root site path** (discussed
above).

### `pages.yml`

This file contains a list of all **pages** in the website. Right now, the list of pages should all be listed under a
top-level `pages` key. Each page can contain the following:

| Key              | Required? | Description                                                                                                                                                                                                                                                               |
|------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `file_path`      | Yes       | The path (relative to `pages_path` found in `server.yml`) to the HTML, Markdown or plain text content for this page.                                                                                                                                                      |
| `title`          | Yes       | The title of the page. This is what will be visible on the website itself.                                                                                                                                                                                                |
| `url`            | Yes       | The URL this page can be accessed at. This is just the path component of the URL, e.g. `/my-page/`.                                                                                                                                                                       |
| `alternate_urls` | No        | A list of alternate URLs this page can be accessed at. If provided, each of these URLs will result in a redirect response to the main page URL. This is provided mainly as an aide in transitioning from another website which may have served content at different URLs. |

An example file may look like the following:

```yml
pages:

  - file_path: about.md
    title: About This Site
    url: /about/

  - file_path: joke.md
    title: Joke
    url: /joke/
    alternate_urls:
      - /trying-to-be-funny/
```

### `posts.yml`

This file contains a list of all **posts** in the website, as well as optional RSS feed configuration.

Each post should be listed under a top-level `posts` key. Each post can contain the following:

| Key              | Required? | Description                                                                                                                                                                                                                                                                                                                           |
|------------------|-----------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `file_path`      | Yes       | The path (relative to `posts_path` found in `server.yml`) to the HTML, Markdown or plain text content for this post.                                                                                                                                                                                                                  |
| `title`          | Yes       | The title of the post. This is what will be visible on the website itself.                                                                                                                                                                                                                                                            |
| `date`           | Yes       | The date/time of the post. This can be written in either `YYYY-MM-DD`, `YYYY-MM-DD HH:MM`, or `YYYY-MM-DD HH:MM:SS` format. If a time is not provided, midnight is assumed internally (when relevant). The date/time of the post is used for sorting as well as for generating the URL to this post (see below for more information). |
| `slug`           | Yes       | The "slug" which is only used when generating the URL for this post (see below for more information).                                                                                                                                                                                                                                 |
| `tags`           | No        | A list of tags for this post. Tagging a post is used for grouping or categorization. Clicking on a tag on the website will show all other posts with the same tag.                                                                                                                                                                    |
| `alternate_urls` | No        | A list of alternate URLs this post can be accessed at. If provided, each of these URLs will result in a redirect response to the main post URL. This is provided mainly as an aide in transitioning from another website which may have served content at different URLs.                                                             |

If you wish to include an RSS feed for your website's posts, you may configure it under the optional `rss` key. The
available keys that can be used here are:

| Key           | Required? | Description                                                                                          |
|---------------|-----------|------------------------------------------------------------------------------------------------------|
| `title`       | Yes       | The title of the RSS feed.                                                                           |
| `description` | Yes       | A short description of the RSS feed (which is effectively just a description of your site, I guess). |
| `url`         | Yes       | The fully qualified public URL to your site. e.g. `http://www.yourdomain.com/`                       |
| `count`       | Yes       | The number of posts to include in the RSS feed. e.g. `10`.                                           |

An example file may look like the following:

```yml
posts:

  - file_path: 2023-01-01-hello-world.md
    title: Hello, world!
    date: 2023-01-01 12:30:42
    slug: hello-world
    tags:
      - aaa
      - hello
      - testing

  - file_path: 2023-02-01-markdown-testing.md
    title: Markdown Testing
    date: 2023-02-01
    slug: markdown-testing
    tags:
      - testing
    alternate_urls:
      - /testing/markdown/

  - file_path: 2023-03-20-lorem-ipsum.md
    title: Lorem Ipsum
    date: 2023-03-20 18:01
    slug: lorem-ipsum

rss:
  title: My Site
  description: This is my site. There are others like it, but this one is mine.
  url: https://www.mydomain.com/
  count: 10
```

#### Post URLs

Post URLs are automatically derived from a combination of the `date` and `slug` defined for each post using the format
`/year/month/day/slug/`.

For example for the post

```yml
file_path: 2023-03-20-lorem-ipsum.md
title: Lorem Ipsum
date: 2023-03-20 18:01
slug: lorem-ipsum
```

The URL would end up being `/2023/03/20/lorem-ipsum/` (note the trailing slash).

## Writing Content

To write content for either a post or page, you simply need to add a new file under the path(s) specified by the
`pages_path` and `posts_path` keys in your `server.yml`.

* **Markdown / CommonMark** content should be saved to files using an `.md` extension.
* **HTML** content should be shaved to files using either an `.html` or `.htm` extension.
* Anything else can use whatever file extension you like.

Markdown/CommonMark content will be parsed and finally rendered out as HTML. This content can also contain HTML
embedded in the Markdown itself as needed.

HTML and all other content will be rendered out to the page as-is.

> TODO: In the future there might be some changes here, such as treating all other content as plain-text and always
> forcing it to be rendered as such, possibly within a forced `<pre>...</pre>` or similar.

## HTML Templates

PBE websites are rendered to HTML via HTML templates, within which the content from your posts and pages are inserted
into and then finally rendered. HTML templates are rendered via [Tera](https://tera.netlify.app/). As such, you can
use any of the features found in their [documentation](https://tera.netlify.app/docs/) in your HTML templates here with
PBE.

These templates are found in the path specified by the `templates_path` key in your `server.yml`. Take a look at the
templates in the example site found in this repository under `/example-site/templates` for examples of what HTML
templates may look like in a PBE site.

Tera lets you compose templates together with some basic [inheritance](https://tera.netlify.app/docs/#inheritance) which
may be useful to you. You don't have to use this feature, but it is useful to include a consistent website look and 
feel across the site.

However, PBE at a minimum requires the templates detailed below.

> **NOTE:** The data available to each template will likely be expanded somewhat in the future to allow for more
> customization.

### `post.html`

Displays any single post at the individual post's URL. Normally this would display the post title, date, its tags and 
the content.

| Key    | Type   | Description |
|--------|--------|-------------|
| `post` | `Post` | The post.   |

### `page.html`

Displays any single page at the individual page's URL. Normally this would display the page title and content.

| Key    | Type   | Description |
|--------|--------|-------------|
| `page` | `Page` | The page.   |

### `tag.html`

Displays posts for a given tag, at the tag's URL `/tag/{tag-name}/`. Normally this would display the tag and then all 
the posts in a list format.

| Key     | Type     | Description                                                              |
|---------|----------|--------------------------------------------------------------------------|
| `posts` | `Post[]` | A list of all posts for the tag, pre-sorted by date in descending order. |
| `tag`   | `string` | The tag.                                                                 |

### `archive.html`

Displays all posts, at the archive URL `/archive/`. Normally this would be a simple list of all posts showing their 
titles, dates, and tags. 

### `latest_post.html`

Displays a single post, the most recent one, at the site's home/main page (that is, the root URL `/`). Normally this 
would look very similar to (if not completely identical to) the `post.html` template. This is provided as a separate 
template since it is used for the home/main page, so you can customize it differently if desired. 

| Key    | Type   | Description           |
|--------|--------|-----------------------|
| `post` | `Post` | The most recent post. |

### Description of HTML Template Data Structures

#### `Post`

Contains all information about a single post.

| Field          | Type       | Description                                                                                                                                                                                                |
|----------------|------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `url`          | `string`   | The post's URL, e.g. `/2023/06/30/hello-world/`                                                                                                                                                            |
| `title`        | `string`   | The post's title, as defined in `posts.yml`.                                                                                                                                                               |
| `date`         | `int`      | The date/time of the post, as defined in `posts.yml`, converted to seconds since Jan 1, 1970. You can use [Tera's `date` filter](https://tera.netlify.app/docs/#date) to display this in a formatted way.  |
| `tags`         | `string[]` | The post's tags, as defined in `posts.yml`. This may be an empty list if no tags were specified for the post.                                                                                              |
| `content_html` | `string`   | The post's content, rendered as HTML. Most of the time, you'd want to display this in your template using [Tera's `safe` filter](https://tera.netlify.app/docs/#safe) to ensure HTML tags are not escaped. |

#### `Page`

Contains all information about a single page.

| Field          | Type     | Description                                                                                                                                                                                                |
|----------------|----------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `url`          | `string` | The page's URL, as defined in `pages.yml`, e.g. `/my-page/`                                                                                                                                                |
| `title`        | `string` | The page's title, as defined in `pages.yml`.                                                                                                                                                               |
| `content_html` | `string` | The page's content, rendered as HTML. Most of the time, you'd want to display this in your template using [Tera's `safe` filter](https://tera.netlify.app/docs/#safe) to ensure HTML tags are not escaped. |

## Caching and Automatic Reloading

PBE internally tries to cache as much configuration and content as it can (with the exception of everything inside the
`static_files_path`). This means changes to post/page content, updates to `pages.yml` or `posts.yml` or any of your
HTML templates, will not be reflected immediately when you reload the browser as the content is not being served from
the files on disk directly.

When PBE starts up, it loads the configuration and content, pre-renders everything and keeps it in memory as a cache.
Whenever page requests are served to visitors, they are served from this internal cache.

However, PBE monitors certain files and directories for changes and will reload itself as needed (after a short roughly 
one or two second delay). These are:

* `pages.yml`
* `posts.yml`
* All files inside the `pages_path`, `posts_path`, and `templates_path` directories, as specified in `server.yml`.

Note that this list **does not** include `server.yml` or the `static_files_path`. Anything inside the `static_files_path`
is always served directly from the files on disk and is not cached by PBE.

---

## Additional Information

### Markdown and Syntax Highlighted Code Blocks

The Markdown/CommonMark renderer used here utilizes [syntect](https://github.com/trishume/syntect) to apply syntax
coloured-highlighting to code blocks included in the Markdown content, but **only if the code block is annotated with
a language/syntax**.

For example this would be parsed and rendered as highlighted HTML:

```text
\```c
#include <stdio.h>

int main(int argc, char *argv[]) {
	printf("Hello, world!\n");
	return 0;
}
\```
```

But this would **not**:
```text
\```
#include <stdio.h>

int main(int argc, char *argv[]) {
	printf("Hello, world!\n");
	return 0;
}
\```
```

Note the lack of `c` annotating the code block to indicate to the highlighter what syntax to use. Auto-detection of the
syntax is not enabled currently.

**The syntaxes are matched using the _file extensions_ defined in the syntax/language files.**

By default, PBE will include all the default syntax/language definitions that syntect ships with, which is the
default bundle that ships with [Sublime Text](https://www.sublimetext.com/), which gives you a great many options out 
of the box.

#### Custom Syntax/Language Definitions

If you've specified the `syntaxes_path` key in your `server.yml` you can place any `.sublime-syntax` files under this
directory, and they will be loaded by PBE and made available to the highlighter.

Note that you cannot use `.tmLanguage` files. They must first be converted to `.sublime-syntax` format. You can use
[this tool](https://github.com/aziz/SublimeSyntaxConvertor) to do this if needed.

#### Syntax Highlighting CSS Styles

Here's where it gets a bit more tricky. Syntax highlighted code blocks are rendered out with a bunch of `<span>` tags
that reference CSS classes named after the particular element of each part of the code (e.g. keyword, function name, 
etc). This requires you to have a CSS sheet with matching class definitions. There are a _lot_ of CSS classes that
can be emitted in rendered highlighted blocks.

The syntect library includes some utility functions for generating CSS sheets from Sublime Text themes, specifically
those in `.tmTheme` format (`.sublime-theme` format themes are not supported).

I've written a quick CLI utility to expose this functionality in syntect to make it easier to use as an end-user. This
can be found in the "syntax_to_css" project within this repository. This tool will let you turn your `.tmTheme` files
from Sublime Text into `.css` files which you can then use with PBE to style your syntax highlighted code blocks.

---

## Not-So-Frequently Asked Questions

### Why not use a static site generator instead?

If you want to use a static site generator, then that is what you should use! Absolutely!

I find it a little boring personally, and I don't really care to learn how to use someone else's project for that.
You'll probably never see me use a static site generator for my own personal use.
