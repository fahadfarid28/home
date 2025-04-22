---
title: "Templating"
date: 2025-03-13T07:00:20Z
---

Templates live in the `templates/` directory:

```term
<i class="b fg-cyn">~/bearcove/home-from-scratch</i>
<i class="b fg-grn">❯</i> <i class="b u fg-blu">ls </i><i class="b u fg-cyn">templates/</i>
page.html.jinja  <i class="b fg-blu">shortcodes</i>
```

Templates _on disk_ have the `.jinja` extension, to:

  * Emphasize that they're using the [Jinja](https://jinja.palletsprojects.com/en/stable/) templating language
  * Enable syntax highlighting in supported editors.

+++
:figure:
    src: jinja-highlighting@2x.jxl
    title: |
        The [Zed](https://zed.dev) code editor supports Jinja syntax highlighting.

        I'm sure VS Code does too.
    alt: |
        A screenshot of the Zed code editor, with some jinja template opened.
+++

## Macros

Via [minijinja](https://lib.rs/crates/minijinja), you have the full power of jinja available, including defining macros:

```jinja
{% macro youtube_embed(id, alt="YouTube Video Thumbnail") %}
<div class="youtube-thumbnail-link paragraph-like">
    <a href="https://www.youtube.com/watch?v={{ id }}{{ extra or '' }}" target="_blank" rel="noopener" class="noclip" data-youtube-id="{{ id }}" data-extra="{{ extra or '' }}">
        <div class="thumbnail-container" style="position: relative; padding-bottom: 56.25%; height: 0; overflow: hidden;">
            <img
                src="https://img.youtube.com/vi/{{ id }}/maxresdefault.jpg"
                alt="{{ alt | escape_for_attribute }}"
                style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: cover; overflow: hidden; border-radius: 8px;"
                onerror="this.onerror=null; this.src='https://img.youtube.com/vi/{{ id }}/0.jpg';"
            />
            <div class="play-button" style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); width: 68px; height: 48px; background-color: rgba(0,0,0,0.7); border-radius: 8px; display: flex; justify-content: center; align-items: center;">
                <div style="width: 0; height: 0; border-style: solid; border-width: 10px 0 10px 20px; border-color: transparent transparent transparent #fff;"></div>
            </div>
        </div>
    </a>
</div>
{% endmacro %}
```

## Defining shortcodes

Shortcodes are just templates defined in `templates/shortcodes`.

There are two shortcodes that home kind of expects: `templates/shortcodes/media.html.jinja`:

```jinja
<p>
{%- set title_attr = title | escape_for_attribute -%}
{%- set alt_attr = alt | escape_for_attribute -%}
{{- get_media(src).markup(title=title_attr, alt=alt_attr, width=width, height=height, class=class) -}}
</p>
```

And `templates/shortcodes/figure.html.jinja`:

```jinja
<figure>
{%- set title_attr = title | escape_for_attribute -%}
{%- set alt_attr = alt | escape_for_attribute -%}
{{- get_media(src).markup(title=title_attr, alt=alt_attr, width=width, height=height, class=class) -}}
<figcaption>{{ title | basic_markdown | safe }}</figcaption>
</figure>
```

You can make those as fancy as you want.

You can import and call macros from shortcodes, for example,
`templates/shortcodes/youtube.html.jinja` could be:

```jinja
{% import "macros.html" as macros %}
{{ macros.youtube_embed(id, alt=alt) }}
```

## Invoking shortcodes

There are two ways to invoke shortcodes, depending if they have a body
or not.

On <https://fasterthanli.me>, the `bearsays` shortcode is defined as:

```jinja
{% import "macros.html" as macros %}

{% if mood is not defined %}
{% set mood = "neutral" %}
{% endif %}

<div class="dialog">
<div class="dialog-head" title="Cool bear says:">
  {{ get_media("/content/img/reimena/cool-bear-" ~ mood ~ ".jxl").markup(width=42, height=42, alt="Cool bear") }}
</div>
<div class="dialog-text markup-container">
{{ body }}
</div>
</div>
```

Notice `{{ body }}` — it's invoked like so:

```markdown
> *:bearsays*
>
> It looks like this!
```

## Main Template Types

Here are the main types you'll be using in templates. They're pretty straightforward, but let's go through some examples.

### LoadedPage

A single page on the site. Could be an article, a series part, whatever.

Properties:
- `path` (String): Content path of the page
- `route` (String): URL route for the page
- `url` (String): Full URL to the page
- `title` (String): Page title
- `html` (HTML String): Full HTML content
- `html_until_playwall` (HTML String): HTML content up to the paywall marker
- `html_until_more` (HTML String): HTML content up to a `<!-- more -->` marker
- `plain_text` (String): Plain text content without HTML
- `short_desc` (String): Truncated description for meta tags
- `date` ([DateTime](#datetime)): Publication date
- `updated_at` ([DateTime](#datetime), optional): Last update date
- `reading_time` (Number): Estimated reading time in minutes
- `tags` (Array of String): Tags associated with the page
- `draft` (Boolean): Whether the page is a draft
- `archive` (Boolean): Whether the page is archived
- `thumb` ([MediaVal](#mediaval), optional): Thumbnail image
- `parent_thumb` ([MediaVal](#mediaval), optional): Parent page's thumbnail
- `toc` (Array): Table of contents entries
- `series_link` (Object, optional): Information about series, if page is part of one
- `crates` (Array): Referenced Rust crates
- `github_repos` (Array): Referenced GitHub repositories
- `links` (Array): External links referenced
- `is_old` (Boolean): True if the page is over two years old
- `exclusive_until` ([DateTime](#datetime), optional): When exclusive content becomes public
- `video_info` (Object): Video-related information

Methods:
- `get_listing(page_number, per_page)`: Returns a [`Listing`](#listing) object with child pages
- `get_children()`: Returns child pages as an array of [`LoadedPage`](#loadedpage) objects

Example:
```jinja
<h1>{{ page.title }}</h1>
<div class="date">{{ page.date | format_day_month_year }}</div>
<div class="content">{{ page.html }}</div>
```

### Listing

A collection of pages. Used for article lists, series, search results, that kinda thing.

Properties:
- `kind` (String): Type of listing ("articles", "episodes", "series", "series-parts")
- `items` (Array of [`LoadedPage`](#loadedpage)): List of page objects
- `page_number` (Number): Current page number
- `per_page` (Number): Items per page
- `has_more` (Boolean): Whether there are more pages

Example:
```jinja
{% for article in listing.items %}
  <h2><a href="{{ article.url }}">{{ article.title }}</a></h2>
{% endfor %}

{% if listing.has_more %}
  <a href="?page={{ listing.page_number + 1 }}">Next page</a>
{% endif %}
```

### MediaVal

A media asset. Usually an image.

Properties:
- `width` (Number): Width in pixels
- `height` (Number): Height in pixels

Methods:
- `markup(width, height, alt, title, id, class)`: Renders HTML markup for the media
- `bitmap_variant_url(codec)`: Returns URL for a specific variant of the media

Example:
```jinja
{{ page.thumb.markup(alt="Thumbnail", width=300) }}
<img src="{{ page.thumb.bitmap_variant_url('webp') }}">
```

### SearchResults

Search results. Pretty self-explanatory.

Properties:
- `results` (Array of [`SearchResult`](#searchresult)): Search result items
- `num_results` (Number): Total number of results
- `terms` (Array of String): Search terms
- `has_more` (Boolean): Whether there are more results

### SearchResult

A single search result.

Properties:
- `page` ([`LoadedPage`](#loadedpage)): The found page
- `title_snippet` (HTML String): Highlighted title snippet
- `body_snippet` (HTML String): Highlighted body snippet

### Globals

Global site info and utilities.

Properties:
- `page` ([`LoadedPage`](#loadedpage), optional): Current page
- `user_info` (Object, optional): Current user information
- `viewer` (Object): Current viewer properties
- `config` (Object): Site configuration
- `sponsors` (Array): Site sponsors

Methods:
- `random_article()`: Returns a random [`LoadedPage`](#loadedpage)
- `get_tag_listing(tag, page_number, per_page)`: Returns a [`Listing`](#listing) for a tag
- `search_page(query, per_page, page_number)`: Returns [`SearchResults`](#searchresults)

Example:
```jinja
{% set random = globals.random_article() %}
<a href="{{ random.url }}">{{ random.title }}</a>
```

### DateTime

A date and time. You'll mostly use it through filters.

Methods exposed through filters:
- `format_day_month_year()`: Returns "Mon DD, YYYY" format
- `format_month_year()`: Returns "Month YYYY" format
- `format_time_ago()`: Returns human-readable relative time
- `format_rfc3339()`: Returns RFC3339 formatted date
- `is_future()`: Returns whether date is in the future

## Built-in Functions

Jinja 2 has a bunch of built-in functions. Here are the ones that Home adds on top:

### `asset_url(path)`

Gets a URL for an asset, with cache busting. Works with relative paths.

```jinja
<link rel="stylesheet" href="{{ asset_url('/content/css/style.css') }}">
```

### `get_media(path)`

Gets a [`MediaVal`](#mediaval) object for a path. Works for images and other media.

```jinja
{{ get_media("/content/images/logo.png").markup(alt="Logo", width=200) }}
```

### `get_recent_pages()`

Gets the 25 most recent published articles and series parts. Useful for RSS feeds.

```jinja
{% for page in get_recent_pages() %}
  <item>
    <title>{{ page.title }}</title>
    <link>{{ page.url }}</link>
    <pubDate>{{ page.date | format_rfc3339 }}</pubDate>
  </item>
{% endfor %}
```

### `url_encode(string)`

Encodes a string for use in URLs.

```jinja
<a href="/search?q={{ url_encode(query) }}">Search results</a>
```

### `html_escape(string)`

Escapes HTML special characters.

```jinja
<div data-content="{{ html_escape(content) }}"></div>
```

### `html_until_playwall`

Render the page until reaching the `<!-- playwall -->` marker

```jinja
{{ page.html_until_playwall }}
```

### `html_until_more`

Render the page until reaching the `<!-- more -->` marker, similar to Zola

```jinja
{{ page.html_until_more }}
```

### `get_page_from_route(route)`

Gets a [`LoadedPage`](#loadedpage) object from a website route.

```jinja
{% set about_page = get_page_from_route("/about") %}
<a href="{{ about_page.url }}">{{ about_page.title }}</a>
```

### `get_page_from_path(path)`

Gets a [`LoadedPage`](#loadedpage) object from a content path.

```jinja
{% set article = get_page_from_path("/content/articles/rust-performance.md") %}
<h2>{{ article.title }}</h2>
```

### `all_icons()`

Gets all available syntax highlighting icons.

```jinja
{% for icon in all_icons() %}
  <i class="icon-{{ icon }}"></i>
{% endfor %}
```

### `basic_markdown(text)`

Renders markdown to HTML.

```jinja
{{ basic_markdown("**Bold text** and _italic text_") | safe }}
```

### `random_article()`

Gets a random Rust-tagged article.

```jinja
{% set random = random_article() %}
<div class="random-recommendation">
  <h3>Random article: <a href="{{ random.url }}">{{ random.title }}</a></h3>
</div>
```

### `get_tag_listing(tag, page_number=1, per_page=25)`

Gets a [`Listing`](#listing) object with paginated content for a specific tag.

```jinja
{% set rust_articles = get_tag_listing(tag="rust", page_number=1, per_page=10) %}
<ul>
  {% for article in rust_articles.items %}
    <li><a href="{{ article.url }}">{{ article.title }}</a></li>
  {% endfor %}
</ul>
{% if rust_articles.has_more %}
  <a href="?page={{ rust_articles.page_number + 1 }}">Next page</a>
{% endif %}
```

### `search_page(query, per_page, page_number)`

Gets a [`SearchResults`](#searchresults) object with pages matching a query.

```jinja
{% set results = search_page(query="Rust performance", per_page=10, page_number=1) %}
<div class="search-results">
  {% for result in results.results %}
    <div class="result">
      <h3><a href="{{ result.page.url }}">{{ result.title_snippet | safe }}</a></h3>
      <p>{{ result.body_snippet | safe }}</p>
    </div>
  {% endfor %}
</div>
```

## Built-in Filters

### `asset_url(path)`

Same as the asset_url function, but as a filter. Gets a URL with cache busting.

```jinja
<img src="{{ '/content/images/logo.png' | asset_url }}">
```

### `url_encode(string)`

Encodes a string for use in URLs.

```jinja
<a href="/search?q={{ query | url_encode }}">Search</a>
```

### `html_escape(string)`

Escapes HTML special characters.

```jinja
<div data-content="{{ content | html_escape }}"></div>
```

### `truncate_html(html, max=300)`

Truncates HTML content while preserving structure. Default limit is 300 characters.

```jinja
{{ article.content | truncate_html(max=150) | safe }}
```

### `truncate(text, len)`

Truncates text to a specified length, adding "..." if truncated.

```jinja
{{ article.description | truncate(len=100) }}
```

### `downcase(string)`

Converts a string to lowercase.

```jinja
<span class="tag">{{ tag | downcase }}</span>
```

### `shuffle(list)`

Randomly shuffles a list.

```jinja
{% for item in items | shuffle %}
  <li>{{ item }}</li>
{% endfor %}
```

### `urlencode(string)`

Encodes a string for use in URLs (same as url_encode).

```jinja
<a href="https://example.com/?q={{ search_term | urlencode }}">Search</a>
```

### `to_json(value)`

Converts a value to pretty-printed JSON.

```jinja
<script>
  const data = {{ page_data | to_json | safe }};
</script>
```

### `basic_markdown(text)`

Renders markdown to HTML.

```jinja
{{ comment.body | basic_markdown | safe }}
```

### `escape_for_attribute(string)`

Escapes a string for use in HTML attributes. Replaces newlines with spaces and double quotes with single quotes.

```jinja
<button title="{{ description | escape_for_attribute }}">More info</button>
```

### `format_time_ago(datetime)`

Formats a date as a human-readable relative time (e.g., "2 days ago").

```jinja
<span class="timestamp">{{ article.date | format_time_ago }}</span>
```

### `format_rfc3339(datetime)`

Formats a date in RFC3339 format.

```jinja
<time datetime="{{ article.date | format_rfc3339 }}">{{ article.date | format_time_ago }}</time>
```

### `format_month_year(datetime)`

Formats a date as "Month Year".

```jinja
<span class="date">{{ article.date | format_month_year }}</span>
```

### `format_day_month_year(datetime)`

Formats a date as "Mon DD, YYYY".

```jinja
<span class="date">{{ article.date | format_day_month_year }}</span>
```

### `is_future(datetime)`

Checks if a date is in the future.

```jinja
{% if article.date | is_future %}
  <span class="badge">Upcoming</span>
{% endif %}
```
