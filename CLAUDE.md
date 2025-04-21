
Dear Claude,

You can find documentations and internal infrastructure in the `docs/` directory.
It's an mdbook. This might be faster than reading a lot of source code.

Remember to update the mdbook in `docs/` when you change behavior!

## Rust Development

After making Rust changes, you can run `cargo check`. If that passes,
you can run `cargo clippy`. Fix those errors by yourself. Thanks!

- Run `cargo check` to verify your changes compile
- No need to run `cargo clean` before checking - incremental compilation is fine
- After `check` passes, run `cargo clippy` to ensure code quality

## `dylo` Module System

An important thing to keep in mind is how dylo modules work — always
modify `crates/mod-config/*.rs` for example, never `crates/config/*.rs`.

The latter is generated from the former by running `dylo gen`. You can run it
yourself after changing anything in `crates/mod-*`. The full documentation
is in `docs/src/architecture/dynamic-modules.md`

Workflow:
1. Edit implementation files in `crates/mod-*/`
2. Run `dylo gen` to generate interface files
3. Run `cargo check` to verify everything works
4. If everything works, run `cargo clippy`

## Branding notes:

Most names are all lowercase. The product is named "home", not "Home".

Similarly, it's lowercase "mom", "cub", etc. Acronyms like "CDN" can be uppercase.
