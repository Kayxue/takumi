use cssparser::{Parser, match_ignore_ascii_case};

use crate::{
  layout::{
    DEFAULT_LINE_HEIGHT_SCALER,
    style::{
      CssToken, FromCss, Length, ParseResult,
      tw::{TW_VAR_SPACING, TailwindPropertyParser},
    },
  },
  rendering::Sizing,
};

/// Represents a line height value, number value is parsed as em.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct LineHeight(pub Length);

impl From<Length> for LineHeight {
  fn from(value: Length) -> Self {
    Self(value)
  }
}

impl Default for LineHeight {
  fn default() -> Self {
    Length::Em(DEFAULT_LINE_HEIGHT_SCALER).into()
  }
}

impl TailwindPropertyParser for LineHeight {
  fn parse_tw(token: &str) -> Option<Self> {
    match_ignore_ascii_case! {&token,
      "none" => Some(Length::Em(1.0).into()),
      "tight" => Some(Length::Em(1.25).into()),
      "snug" => Some(Length::Em(1.375).into()),
      "normal" => Some(Length::Em(1.5).into()),
      "relaxed" => Some(Length::Em(1.625).into()),
      "loose" => Some(Length::Em(2.0).into()),
      _ => {
        let Ok(value) = token.parse::<f32>() else {
          return None;
        };

        Some(Length::Em(value * TW_VAR_SPACING).into())
      }
    }
  }
}

impl<'i> FromCss<'i> for LineHeight {
  fn from_css(input: &mut Parser<'i, '_>) -> ParseResult<'i, Self> {
    let Ok(number) = input.try_parse(Parser::expect_number) else {
      return Length::from_css(input).map(LineHeight);
    };

    Ok(Length::Em(number).into())
  }

  fn valid_tokens() -> &'static [CssToken] {
    &[CssToken::Token("number"), CssToken::Token("length")]
  }
}

impl LineHeight {
  pub(crate) fn into_parley(self, sizing: &Sizing) -> parley::LineHeight {
    parley::LineHeight::Absolute(self.0.to_px(sizing, sizing.font_size))
  }
}
