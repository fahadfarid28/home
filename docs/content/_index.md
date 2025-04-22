---
title: "home"
date: 2025-03-13T06:00:00Z
---

This guide is designed for people who want to author content using home.

home is a cozy content authoring system that resembles
[hugo](https://gohugo.io/) or [zola](https://www.getzola.org/) on the surface,
but comes with several important distinctions.

## home is not a static site generator

Although a lot of things are decided at deploy time, home sites cannot
be served from "dumb" object storage like S3.

Having a server component allows home to provide:

  * "Log in with GitHub" / "Log in with Patreon" functionality
  * A proper search experience, via [tantivy](https://lib.rs/crates/tantivy)
  * Effortless atomic deploys, regardless of the underlying object storage

## home is _very_ opinionated

There's a few things that home makes no compromises on:

  * images are stored as [JPEG-XL](https://jpeg.org/jpegxl/) (or [SVG](https://developer.mozilla.org/en-US/docs/Web/SVG))
  * short videos are stored as [AV1](https://en.wikipedia.org/wiki/AV1)+[Opus](https://opus-codec.org/)
  * scripts are [TypeScript](https://www.typescriptlang.org/), components are [Svelte 5](https://svelte.dev/)
  * styles are [SCSS](https://sass-lang.com/)
  * templates are [Jinja2](https://jinja.palletsprojects.com/)
  * the bundler is [vite](https://vitejs.dev/) — that's it.

In a way, this is kinda relaxing. It's a stack that works. It's all
integrated together. There's no anxiety about making the right choice.

## home is multi-tenant

home is in fact a pair of web services, `mom`, deployed on a dedicated
hetzner server in Germany, and `cub`, deployed on various hetzner VMs
around the world.

`mom` is in charge of receiving deploys, proxying to object storage,
and executing derivations (image/video re-encodes, etc.)

`cub` is in charge of rendering templates, caching assets at the edge
(= geographically close to visitors) in memory, on SSD storage, etc.

All cubs serve all the websites powered by home — which includes:

  * <https://fasterthanli.me>
  * <https://bearcove.eu>
  * <https://home.bearcove.eu>
  * <https://facet.rs>

This is why you can't just "run home on your VPS". I mean, you can, but
it would sorta defeat the purpose — it's designed to have that split,
just like kubernetes has separate servers (control plane) and worker nodes.
