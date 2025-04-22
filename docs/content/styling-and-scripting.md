---
title: "Styling and scripting"
date: 2025-03-13T07:00:30Z
---

home uses the [vite](https://vitejs.dev/) bundler to deal with both styles and scripts.

The Vite configuration along with the `package.json` file and everything is created
automatically when running the `home init` command.

`vite` is started automatically by home, and home proxies requests to it
(including websocket) — so you never need to worry about it at all.

## `home-base`

Most importantly, a dependency on the
[`@bearcove/home-base`](https://www.npmjs.com/package/@bearcove/home-base) NPM
package is set up by `home` when serving a website.

This package contains a set of base styles used by home, for example classes for
syntax highlighting and ANSI terminal colors (using the [catppuccin](https://catppuccin.com) color scheme).

## Entry point

There's a single entry point to both styles and scripts, and it is expected to be a TypeScript file at `src/bundle.ts`.

The simplest entry point looks like this:

```typescript
import { renderAdmin } from "@bearcove/home-base";
import "./main.scss";

if (import.meta.env.DEV) {
    renderAdmin();
}
```

This renders the admin controls from home-base in developments and pulls in SCSS styles from `src/main.scss`,
which should look something like:

```scss
@use "@bearcove/home-base/base.scss" as hb;

// SASS has its own standard library, look it up: <https://sass-lang.com/documentation/modules/>
@use "sass:math";
@use "sass:list";

// those are SASS partials, defined in `src/_mixins.scss`, etc.
@use "mixins";
@use "vars";

html {
    // CSS variables are good, use them
    font-family: var(--font-text);
    background: var(--html-bg);
    color-scheme: light dark;
}

main {
    // etc.
}
```

## Svelte components

The framework chosen by home to provide admin controls is Svelte 5.

`renderAdmin()` mounts a little button in the corner that allows expanding admin
controls and deploying:

+++
:media:
    src: deploying@2x.mp4
    alt: |
        A video that shows how to expand the admin drawer and click on the deploy button, which then shows a build log culminating into a confetti when the deploy is complete.
+++

You can open this panel with `Alt-P` (`⌥-P` on macOS) and once it's open, trigger
a deploy with `Alt-D` (`⌥-D` on macOS).

## Hot module reloading

While the local version of a home website is open in the browser, making changes
to any scripts or styles and saving will attempt to inject those changes in the
browser without reloading the page.

Home also attempts to apply diffs of a page while building it, but it's a work
in progress and is not really working as it should right now.
