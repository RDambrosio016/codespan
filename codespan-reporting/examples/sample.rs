//! Renders the README sample SVG.
//!
//! To update the sample run the following command from the top level of the repository:
//!
//! ```sh
//! cargo run --example sample svg > codespan-reporting/assets/sample.svg
//! ```

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use codespan_reporting::term::{self, ColorArg};
use std::io::{self, Write};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub enum Opts {
    /// Render SVG output
    Svg,
    /// Render Stderr output
    Stderr {
        /// Configure coloring of output
        #[structopt(
            long = "color",
            parse(try_from_str),
            default_value = "auto",
            possible_values = ColorArg::VARIANTS,
            case_insensitive = true
        )]
        color: ColorArg,
    },
}

const SVG_START: &str = r#"<svg viewBox="0 0 882 440" xmlns="http://www.w3.org/2000/svg">
  <style>
    /* https://github.com/aaron-williamson/base16-alacritty/blob/master/colors/base16-tomorrow-night-256.yml */
    pre {
      background: #1d1f21;
      padding: 10px;
      border-radius: 6px;
      color: #ffffff;
      font: 12px SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace;
    }

    pre .bold { font-weight: bold; }

    pre .fg.black   { color: #1d1f21; }
    pre .fg.red     { color: #cc6666; }
    pre .fg.green   { color: #b5bd68; }
    pre .fg.yellow  { color: #f0c674; }
    pre .fg.blue    { color: #81a2be; }
    pre .fg.magenta { color: #b294bb; }
    pre .fg.cyan    { color: #8abeb7; }
    pre .fg.white   { color: #c5c8c6; }

    pre .fg.black.bright    { color: #969896; }
    pre .fg.red.bright      { color: #cc6666; }
    pre .fg.green.bright    { color: #b5bd68; }
    pre .fg.yellow.bright   { color: #f0c674; }
    pre .fg.blue.bright     { color: #81a2be; }
    pre .fg.magenta.bright  { color: #b294bb; }
    pre .fg.cyan.bright     { color: #8abeb7; }
    pre .fg.white.bright    { color: #ffffff; }

    pre .bg.black   { background-color: #1d1f21; }
    pre .bg.red     { background-color: #cc6666; }
    pre .bg.green   { background-color: #b5bd68; }
    pre .bg.yellow  { background-color: #f0c674; }
    pre .bg.blue    { background-color: #81a2be; }
    pre .bg.magenta { background-color: #b294bb; }
    pre .bg.cyan    { background-color: #8abeb7; }
    pre .bg.white   { background-color: #c5c8c6; }

    pre .bg.black.bright    { background-color: #969896; }
    pre .bg.red.bright      { background-color: #cc6666; }
    pre .bg.green.bright    { background-color: #b5bd68; }
    pre .bg.yellow.bright   { background-color: #f0c674; }
    pre .bg.blue.bright     { background-color: #81a2be; }
    pre .bg.magenta.bright  { background-color: #b294bb; }
    pre .bg.cyan.bright     { background-color: #8abeb7; }
    pre .bg.white.bright    { background-color: #ffffff; }
  </style>

  <foreignObject x="0" y="0" width="882" height="440">
    <div xmlns="http://www.w3.org/1999/xhtml">
      <pre>"#;

const SVG_END: &str = "</pre>
    </div>
  </foreignObject>
</svg>
";

fn main() -> anyhow::Result<()> {
    let file = SimpleFile::new(
        "FizzBuzz.fun",
        unindent::unindent(
            r#"
                module FizzBuzz where

                fizz₁ : Nat → String
                fizz₁ num = case (mod num 5) (mod num 3) of
                    0 0 => "FizzBuzz"
                    0 _ => "Fizz"
                    _ 0 => "Buzz"
                    _ _ => num

                fizz₂ : Nat → String
                fizz₂ num =
                    case (mod num 5) (mod num 3) of
                        0 0 => "FizzBuzz"
                        0 _ => "Fizz"
                        _ 0 => "Buzz"
                        _ _ => num
            "#,
        ),
    );

    let diagnostics = [Diagnostic::error()
        .with_message("`case` clauses have incompatible types")
        .with_code("E0308")
        .with_labels(vec![
            Label::primary((), 328..331).with_message("expected `String`, found `Nat`"),
            Label::secondary((), 211..331).with_message("`case` clauses have incompatible types"),
            Label::secondary((), 258..268).with_message("this is found to be of type `String`"),
            Label::secondary((), 284..290).with_message("this is found to be of type `String`"),
            Label::secondary((), 306..312).with_message("this is found to be of type `String`"),
            Label::secondary((), 186..192).with_message("expected type `String` found here"),
        ])
        .with_notes(vec![unindent::unindent(
            "
                    expected type `String`
                       found type `Nat`
                ",
        )])];

    // let mut files = SimpleFiles::new();
    match Opts::from_args() {
        Opts::Svg => {
            let stdout = std::io::stdout();
            let mut writer = SvgWriter::new(stdout.lock());
            let config = codespan_reporting::term::Config::default();

            write!(writer, "{}", SVG_START)?;
            for diagnostic in &diagnostics {
                term::emit(&mut writer, &config, &file, &diagnostic)?;
            }
            write!(writer, "{}", SVG_END)?;
        }
        Opts::Stderr { color } => {
            let writer = StandardStream::stderr(color.into());
            let config = codespan_reporting::term::Config::default();
            for diagnostic in &diagnostics {
                term::emit(&mut writer.lock(), &config, &file, &diagnostic)?;
            }
        }
    }

    Ok(())
}

pub struct SvgWriter<W> {
    upstream: W,
    color: ColorSpec,
}

impl<W> SvgWriter<W> {
    pub fn new(upstream: W) -> SvgWriter<W> {
        SvgWriter {
            upstream,
            color: ColorSpec::new(),
        }
    }
}

impl<W: Write> Write for SvgWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.upstream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.upstream.flush()
    }
}

impl<W: Write> WriteColor for SvgWriter<W> {
    fn supports_color(&self) -> bool {
        true
    }

    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
        #![allow(unused_assignments)]

        if self.color == *spec {
            return Ok(());
        } else {
            if !self.color.is_none() {
                write!(self, "</span>")?;
            }
            self.color = spec.clone();
        }

        if spec.is_none() {
            write!(self, "</span>")?;
            return Ok(());
        } else {
            write!(self, "<span class=\"")?;
        }

        let mut first = true;

        fn write_first<W: Write>(first: bool, writer: &mut SvgWriter<W>) -> io::Result<bool> {
            if !first {
                write!(writer, " ")?;
            }

            Ok(false)
        };

        fn write_color<W: Write>(color: &Color, writer: &mut SvgWriter<W>) -> io::Result<()> {
            match color {
                Color::Black => write!(writer, "black"),
                Color::Blue => write!(writer, "blue"),
                Color::Green => write!(writer, "green"),
                Color::Red => write!(writer, "red"),
                Color::Cyan => write!(writer, "cyan"),
                Color::Magenta => write!(writer, "magenta"),
                Color::Yellow => write!(writer, "yellow"),
                Color::White => write!(writer, "white"),
                // TODO: other colors
                _ => Ok(()),
            }
        };

        if let Some(fg) = spec.fg() {
            first = write_first(first, self)?;
            write!(self, "fg ")?;
            write_color(fg, self)?;
        }

        if let Some(bg) = spec.bg() {
            first = write_first(first, self)?;
            write!(self, "bg ")?;
            write_color(bg, self)?;
        }

        if spec.bold() {
            first = write_first(first, self)?;
            write!(self, "bold")?;
        }

        if spec.underline() {
            first = write_first(first, self)?;
            write!(self, "underline")?;
        }

        if spec.intense() {
            first = write_first(first, self)?;
            write!(self, "bright")?;
        }

        write!(self, "\">")?;

        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        let color = self.color.clone();

        if color != ColorSpec::new() {
            write!(self, "</span>")?;
            self.color = ColorSpec::new();
        }

        Ok(())
    }
}
