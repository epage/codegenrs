//! # codegenrs
//!
//! > **Nitpicking commit history since `beabf39`**
//!
//! ## About
//!
//! `codegenrs` makes it easy to get rid of code-gen in `build.rs`, reducing your
//! and dependents' build times.  This is done by:
//! - Creating a child `[[bin]]` crate that does code-gen using `codegenrs`
//! - Do one-time code-gen and commit it
//! - Run the `--check` step in your CI to ensure your code-gen is neither out of
//!   date or been human edited.
//!
//!## Usage
//!
//!```toml
//![dependencies]
//!codegenners = "0.1"
//!structopt = "0.3"
//!```
//!
//!`imperative` example:
//! - output: [`wordlist_codegen.rs`](https://github.com/crate-ci/imperative/blob/master/src/wordlist_codegen.rs)
//! - generator: [`imperative-codegen`](https://github.com/crate-ci/imperative/tree/master/codegen)
//! - audit: [`azure-pipelines.yml`](https://github.com/crate-ci/imperative/blob/master/azure-pipelines.yml#L13)

use std::io::Write;
use std::iter::FromIterator;

#[cfg(features = "structopt")]
use structopt::StructOpt;

/// CLI arguments to `flatten` into your args
///
/// ## Example
///
/// ```rust
/// #[structopt::StructOpt]
/// struct Args{
///    #[structopt(short("-o"), long, parse(from_os_str))]
///    output: std::path::PathBuf,
///    #[structopt(flatten)]
///    codegen: codegeners::CodeGenArgs,
/// }
/// ```
#[cfg(features = "structopt")]
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct CodeGenArgs {
    #[structopt(short("-o"), long, parse(from_os_str))]
    output: std::path::PathBuf,

    #[structopt(long)]
    check: bool,
}

#[cfg(features = "structopt")]
impl CodeGenArgs {
    /// Write or verify code-genned text.
    fn write_str(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        write_str(content, &self.output, self.check)
    }
}

/// Write or verify code-genned text.
///
/// See `CodeGenArgs` for `structopt` integration.
pub fn write_str(
    content: &str,
    output: &std::path::Path,
    check: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if check {
        let content = String::from_iter(normalize_line_endings::normalized(content.chars()));

        let actual = std::fs::read_to_string(output)?;
        let actual = String::from_iter(normalize_line_endings::normalized(actual.chars()));

        let changeset = difference::Changeset::new(&actual, &content, "\n");
        if changeset.distance != 0 {
            eprintln!("{}", changeset);
            return Err(Box::new(CodeGenError));
        } else {
            println!("Success");
        }
    } else {
        let mut file = std::io::BufWriter::new(std::fs::File::create(output)?);
        write!(file, "{}", content)?;
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, derive_more::Display)]
#[display(fmt = "Code-gen failed")]
struct CodeGenError;

impl std::error::Error for CodeGenError {}
