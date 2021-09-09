use std::fmt::Write;

use mdbook::{
    book::{Book, Chapter},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct MdbookSnipsConfig {
    #[serde(default = "true_f")]
    for_imports: bool,
    #[serde(default)]
    for_end_of_block: bool,
    #[serde(default = "default_snip")]
    snip_text: String,
}

fn true_f() -> bool {
    true
}

fn default_snip() -> String {
    String::from("// --snip--")
}

impl Default for MdbookSnipsConfig {
    fn default() -> Self {
        Self {
            for_imports: true,
            for_end_of_block: false,
            snip_text: default_snip(),
        }
    }
}
pub struct MdbookSnips;

lazy_static::lazy_static! {
    // from https://github.com/rust-lang/mdBook/blob/d22299d9985eafc87e6103974700a3eb8e24d73d/src/renderer/html_handlebars/hbs_renderer.rs#L886
    static ref BORING_LINES_REGEX: Regex = Regex::new(r"^(\s*)#(.?)(.*)$").unwrap();

    static ref IS_USE_STMT: Regex = Regex::new(
        r#"^\s*(pub(\s+|\s*\((crate|in\s+(::)?[a-zA-Z_][a-zA-Z0-9_]*(::[a-zA-Z_][a-zA-Z0-9_]*)*)\)\s*))?use\s+"#
    ).unwrap();
}

fn space_width(ch: char) -> usize {
    // TODO: not all chars have 1-space width, and tabs are annoying
    match ch {
        ' ' => 1,
        '\t' => 4, // FIXME: do tabs properly
        _ => unimplemented!(),
    }
}

impl MdbookSnips {
    pub fn new() -> MdbookSnips {
        MdbookSnips
    }

    fn handle_content(
        &self,
        cfg: &MdbookSnipsConfig,
        content: &mut String,
    ) -> Result<(), std::fmt::Error> {
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
        enum InCodeState {
            Rust,
            Other,
            None,
        }

        #[derive(Copy, Clone, Debug)]
        struct BoringBlockData {
            start_line_index: usize,
            indentation_spaces: usize,
            is_import_block: bool, // starts with an "use"
        }

        let mut in_code = InCodeState::None;
        let mut current_block = None;
        let mut boring_blocks = vec![]; // line indices to add `// --snip--` before
        for (line_ind, line) in content.lines().enumerate() {
            if line == "```rust" || line.starts_with("```rust ,") || line.starts_with("```rust,") {
                in_code = InCodeState::Rust;
            } else if line == "```" {
                if in_code == InCodeState::Rust && cfg.for_end_of_block {
                    if let Some(block) = current_block {
                        boring_blocks.push(block);
                    }
                }

                in_code = match in_code {
                    InCodeState::Rust | InCodeState::Other => InCodeState::None,
                    InCodeState::None => InCodeState::Rust,
                };
            } else if line.starts_with("```") {
                in_code = match in_code {
                    InCodeState::Rust => {
                        eprintln!(
                            "mdbook-snips: Probably invalid block ending for rust block: {:?}",
                            line
                        );
                        InCodeState::None
                    }
                    InCodeState::None => InCodeState::Other,
                    InCodeState::Other => {
                        eprintln!(
                            "mdbook-snips: Probably invalid block ending for non-rust block: {:?}",
                            line
                        );
                        InCodeState::None
                    }
                }
            }

            if in_code != InCodeState::Rust {
                current_block = None;
                continue;
            }

            if BORING_LINES_REGEX.is_match(line) {
                if current_block.is_none() {
                    let boring_line = &line.trim_start()[1..];
                    // protect against attribute and derive macros
                    if !boring_line.starts_with('[') && !boring_line.starts_with("![") {
                        let boring_line = boring_line.strip_prefix(" ").unwrap_or(boring_line);
                        let before_hash_whitespace_chars = line
                            .chars()
                            .take_while(|it| it.is_whitespace())
                            .map(space_width)
                            .sum::<usize>();
                        let after_hash_whitespace_chars = boring_line
                            .chars()
                            .take_while(|it| it.is_whitespace())
                            .map(space_width)
                            .sum::<usize>();
                        let whitespace_chars =
                            before_hash_whitespace_chars + after_hash_whitespace_chars;
                        let skip_ws = boring_line.trim_start_matches(char::is_whitespace);
                        let is_import = IS_USE_STMT.is_match(skip_ws);
                        current_block = Some(BoringBlockData {
                            start_line_index: line_ind,
                            indentation_spaces: whitespace_chars,
                            is_import_block: is_import,
                        });
                    }
                }
            } else {
                if let Some(block) = current_block {
                    if block.is_import_block {
                        if cfg.for_imports {
                            boring_blocks.push(block);
                        }
                    } else {
                        boring_blocks.push(block);
                    }

                    current_block = None;
                }
            }
        }

        if boring_blocks.is_empty() {
            // no "boring blocks" that we have to act on, just return without changing anything
            return Ok(());
        }

        let old_content = std::mem::replace(content, String::new());

        let mut current_boring_block_index = 0_usize;
        for (line_ind, line) in old_content.lines().enumerate() {
            if current_boring_block_index < boring_blocks.len()
                && line_ind == boring_blocks[current_boring_block_index].start_line_index
            {
                let indent = boring_blocks[current_boring_block_index].indentation_spaces;
                writeln!(content, "{:indent$}{}", "", cfg.snip_text, indent = indent)?;

                current_boring_block_index += 1;
            }

            writeln!(content, "{}", line)?;
        }

        Ok(())
    }

    fn handle_chapter_content(
        &self,
        cfg: &MdbookSnipsConfig,
        chapter: &mut Chapter,
    ) -> Result<(), std::fmt::Error> {
        self.handle_content(cfg, &mut chapter.content)
    }

    fn handle(&self, cfg: &MdbookSnipsConfig, item: &mut BookItem) -> Result<(), std::fmt::Error> {
        match item {
            BookItem::Chapter(chapter) => {
                self.handle_chapter_content(cfg, chapter)?;
            }
            BookItem::Separator => {}
            BookItem::PartTitle(_) => {}
        }

        Ok(())
    }
}

impl Preprocessor for MdbookSnips {
    fn name(&self) -> &str {
        "mdbook-snips"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let config = ctx
            .config
            .get_preprocessor(self.name())
            .and_then(|it| {
                Some(MdbookSnipsConfig::deserialize(toml::Value::Table(it.clone())).unwrap())
            })
            .unwrap_or_default();
        book.for_each_mut(|item| {
            self.handle(&config, item).unwrap_or_else(|e| {
                eprintln!("mdbook-snips: {}", e);
                std::process::exit(1)
            })
        });

        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}

#[cfg(test)]
mod tests;
