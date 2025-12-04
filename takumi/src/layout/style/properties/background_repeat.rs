use cssparser::{Parser, Token, match_ignore_ascii_case};
use smallvec::SmallVec;

use crate::layout::style::{FromCss, ParseResult};

/// Per-axis repeat style.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BackgroundRepeatStyle {
  /// Tile as many times as needed with no extra spacing
  #[default]
  Repeat,
  /// Do not tile on this axis
  NoRepeat,
  /// Distribute leftover space evenly between tiles; edges flush with sides
  Space,
  /// Scale tile so an integer number fits exactly
  Round,
}

impl<'i> FromCss<'i> for BackgroundRepeatStyle {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! {
      &ident,
      "repeat" => Ok(BackgroundRepeatStyle::Repeat),
      "no-repeat" => Ok(BackgroundRepeatStyle::NoRepeat),
      "space" => Ok(BackgroundRepeatStyle::Space),
      "round" => Ok(BackgroundRepeatStyle::Round),
      _ => Err(location.new_basic_unexpected_token_error(Token::Ident(ident.clone())).into()),
    }
  }
}

/// Combined repeat for X and Y axes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BackgroundRepeat(pub BackgroundRepeatStyle, pub BackgroundRepeatStyle);

impl BackgroundRepeat {
  /// Returns a repeat value that tiles on both the X and Y axes.
  pub const fn repeat() -> Self {
    Self(BackgroundRepeatStyle::Repeat, BackgroundRepeatStyle::Repeat)
  }

  /// Returns a repeat value that does not tile on either axis.
  pub const fn no_repeat() -> Self {
    Self(
      BackgroundRepeatStyle::NoRepeat,
      BackgroundRepeatStyle::NoRepeat,
    )
  }

  /// Returns a repeat value that distributes leftover space evenly between tiles; edges flush with sides.
  pub const fn space() -> Self {
    Self(BackgroundRepeatStyle::Space, BackgroundRepeatStyle::Space)
  }

  /// Returns a repeat value that scales tile so an integer number fits exactly.
  pub const fn round() -> Self {
    Self(BackgroundRepeatStyle::Round, BackgroundRepeatStyle::Round)
  }
}

impl<'i> FromCss<'i> for BackgroundRepeat {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let state = input.state();
    let ident = input.expect_ident()?;

    match_ignore_ascii_case! { ident,
      "repeat-x" => return Ok(BackgroundRepeat(BackgroundRepeatStyle::Repeat, BackgroundRepeatStyle::NoRepeat)),
      "repeat-y" => return Ok(BackgroundRepeat(BackgroundRepeatStyle::NoRepeat, BackgroundRepeatStyle::Repeat)),
      _ => {}
    }

    input.reset(&state);

    let x = BackgroundRepeatStyle::from_css(input)?;
    let y = input
      .try_parse(BackgroundRepeatStyle::from_css)
      .unwrap_or(x);
    Ok(BackgroundRepeat(x, y))
  }
}

/// A list of background-repeat values (one per layer).
pub type BackgroundRepeats = SmallVec<[BackgroundRepeat; 4]>;

impl<'i> FromCss<'i> for BackgroundRepeats {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut values = SmallVec::new();
    values.push(BackgroundRepeat::from_css(input)?);

    while input.expect_comma().is_ok() {
      values.push(BackgroundRepeat::from_css(input)?);
    }

    Ok(values)
  }
}
