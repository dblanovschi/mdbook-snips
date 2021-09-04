# `mdbook-snips`

A [mdbook](https://github.com/rust-lang/mdBook/) preprocessor to add `// --snip--`
(or similar) before all "blocks" of hidden lines in rust blocks in a mdbook book,
making it very clear that there is some hidden code there.

(mdbook calls them boring lines).

Example:

Before:
```rust
# fn f();
#
fn g();

fn main() {
#     f();
    g();
}
```

After:
```rust
// --snip--
# fn f();
# 
fn g();

fn main() {
    // --snip--
#     f();
    g();
}
```

Usage:

```bash
cargo install mdbook-snips
```

Then in the `book.toml` of your book:

```toml
[preprocessors.mdbook-snips]
command = "mdbook-snips"
```

Default configuration:
```toml
# book.toml
[preprocessors.mdbook-snips]
command = "mdbook-snips"
for_imports = true
for_end_of_block = false
snip_text = "// --snip--"
```

- `for_imports`
Emits a `// --snip--` if the first line of the boring "block" is an import
(e.g. starts with the exact string `"use "`)

- `for_end_of_block`
Emits a `// --snip--` if the last line of the boring "block" ends on the
last line of syntax highlighting block, e.g. for:

```
\```rust
fn main() {

}

# fn f() {}
\```
```

With `for_end_of_block=true`, it ends up as:

```
\```rust
fn main() {

}

// --snip--
# fn f() {}
\```
```

But with `for_end_of_block=false`, it doesn't change:
```
\```rust
fn main() {

}

# fn f() {}
\```
```

- `snip_text`
If you want to change the `// --snip--` text to something else, you can.

For example, you can use a block comment:
`snip_text="/* --snip-- */"`

Which, for:

```
# fn f() {}

fn main() {}
```

Will give you:

```
/* --snip-- */
# fn f() {}

fn main() {}
```