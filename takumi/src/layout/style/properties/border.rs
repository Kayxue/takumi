use cssparser::{Parser, ParserInput, Token};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::layout::style::{
  FromCss, ParseResult,
  properties::{Color, LengthUnit},
};

/// Represents the `border` shorthand which accepts a width, style ("solid"), and an optional color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub(crate) enum BorderValue {
  /// Structured representation when provided as JSON.
  #[serde(rename_all = "camelCase")]
  Structured {
    width: Option<LengthUnit>,
    style: Option<BorderStyle>,
    color: Option<Color>,
  },
  /// Raw CSS string representation.
  Css(String),
}

/// Represents border style options (currently only solid is supported).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum BorderStyle {
  /// Solid border style.
  Solid,
}

/// Parsed `border` value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(try_from = "BorderValue")]
#[ts(as = "BorderValue")]
pub struct Border {
  /// Border width as a `LengthUnit`.
  pub width: Option<LengthUnit>,
  /// Border style (currently only solid is supported).
  pub style: Option<BorderStyle>,
  /// Optional border color.
  pub color: Option<Color>,
}

impl TryFrom<BorderValue> for Border {
  type Error = String;

  fn try_from(value: BorderValue) -> Result<Self, Self::Error> {
    match value {
      BorderValue::Structured {
        width,
        style,
        color,
      } => Ok(Border {
        width,
        style,
        color,
      }),
      BorderValue::Css(s) => {
        let mut input = ParserInput::new(&s);
        let mut parser = Parser::new(&mut input);

        Ok(Border::from_css(&mut parser).map_err(|e| e.to_string())?)
      }
    }
  }
}

impl<'i> FromCss<'i> for Border {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let mut width = None;
    let mut style = None;
    let mut color = None;

    loop {
      if let Ok(value) = input.try_parse(LengthUnit::from_css) {
        width = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(BorderStyle::from_css) {
        style = Some(value);
        continue;
      }

      if let Ok(value) = input.try_parse(Color::from_css) {
        color = Some(value);
        continue;
      }

      if input.is_exhausted() {
        break;
      }

      let location = input.current_source_location();
      let token = input.next()?;

      return Err(
        location
          .new_basic_unexpected_token_error(token.clone())
          .into(),
      );
    }

    Ok(Border {
      width,
      style,
      color,
    })
  }
}

impl<'i> FromCss<'i> for BorderStyle {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let location = input.current_source_location();
    let token = input.next()?;

    if let Token::Ident(ident) = token
      && ident.eq_ignore_ascii_case("solid")
    {
      return Ok(BorderStyle::Solid);
    }

    Err(
      location
        .new_basic_unexpected_token_error(token.clone())
        .into(),
    )
  }
}
