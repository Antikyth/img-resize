use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

/// The name of the command.
pub const NAME: &str = "img-resize";

#[derive(Debug, PartialEq, Eq, Clone, Parser)]
#[command(name = NAME, args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Option<Subcommands>,

    #[command(flatten)]
    pub resize_args: Option<ResizeArgs>,
}

#[derive(Debug, PartialEq, Eq, Clone, Parser)]
pub struct ResizeArgs {
    /// The image to extend.
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub input_path: String,

    /// The output image path - will overwrite the `input_path` if not provided.
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub output_path: Option<String>,

    /// The scale (arbitrary units) of the image currently.
    #[arg(long, value_name = "WIDTH x HEIGHT")]
    pub fit_scale: Size<u32>,
    /// The scale to extend the image to, in relation to `fit_scale`.
    #[arg(long, value_name = "WIDTH x HEIGHT")]
    pub output_scale: Size<u32>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Subcommand)]
pub enum Subcommands {
    /// Generates shell completions for the given shell.
    Generate {
        #[arg(short, long, value_name = "SHELL")]
        shell: Shell,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Size<T>(pub T, pub T);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SizeError<ParseErr> {
    MissingSeparator(String),
    DimensionParseError(ParseErr),
}

impl<ParseErr: Display> Display for SizeError<ParseErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSeparator(string) => {
                write!(f, "invalid size: no 'x' separator found in {string}")
            }

            Self::DimensionParseError(error) => error.fmt(f),
        }
    }
}

impl<ParseErr: Error + 'static> Error for SizeError<ParseErr> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::MissingSeparator(_) => None,
            Self::DimensionParseError(error) => Some(error),
        }
    }
}

impl<T: FromStr> FromStr for Size<T>
where
    T::Err: 'static,
{
    type Err = SizeError<T::Err>;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Find the separator.
        let separator_pos = string
            .find(|char| matches!(char, 'x'))
            .ok_or_else(|| SizeError::MissingSeparator(string.to_owned()))?;

        // Parse the width and height.
        let width = string[..separator_pos]
            .parse()
            .map_err(SizeError::DimensionParseError)?;
        let height = string[separator_pos + 1..]
            .parse()
            .map_err(SizeError::DimensionParseError)?;

        Ok(Size(width, height))
    }
}
