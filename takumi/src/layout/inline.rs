use std::borrow::Cow;

use parley::InlineBox;
use taffy::{AvailableSpace, Size};

use crate::{
  GlobalContext,
  layout::{
    node::Node,
    style::{Color, SizedFontStyle, TextWrapStyle},
  },
  rendering::{
    MaxHeight, RenderContext, apply_text_transform, apply_white_space_collapse, make_balanced_text,
    make_pretty_text,
  },
};

pub(crate) struct InlineNodeItem<'c, 'g, N: Node<N>> {
  pub(crate) node: &'c N,
  pub(crate) context: &'c RenderContext<'g>,
}

pub(crate) enum InlineItem<'c, 'g, N: Node<N>> {
  Node(InlineNodeItem<'c, 'g, N>),
  Text {
    text: Cow<'c, str>,
    context: &'c RenderContext<'g>,
  },
}

pub enum InlineContentKind {
  Text(String),
  Box,
}

pub type InlineLayout = parley::Layout<InlineBrush>;

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct InlineBrush {
  pub color: Color,
  pub decoration_color: Color,
  pub stroke_color: Color,
}

impl Default for InlineBrush {
  fn default() -> Self {
    Self {
      color: Color::black(),
      decoration_color: Color::black(),
      stroke_color: Color::black(),
    }
  }
}

pub(crate) fn measure_inline_layout(
  layout: &mut InlineLayout,
  max_width: f32,
  max_height: Option<MaxHeight>,
) -> Size<f32> {
  break_lines(layout, max_width, max_height);

  let (max_run_width, total_height) =
    layout
      .lines()
      .fold((0.0, 0.0), |(max_run_width, total_height), line| {
        let metrics = line.metrics();
        (
          metrics.advance.max(max_run_width),
          total_height + metrics.line_height,
        )
      });

  Size {
    width: max_run_width.ceil().min(max_width),
    height: total_height.ceil(),
  }
}

pub(crate) fn create_inline_layout<'c, 'g: 'c, N: Node<N> + 'c>(
  items: impl Iterator<Item = InlineItem<'c, 'g, N>>,
  available_space: Size<AvailableSpace>,
  max_width: f32,
  max_height: Option<MaxHeight>,
  style: &SizedFontStyle,
  global: &'_ GlobalContext,
  measure_only: bool,
) -> (InlineLayout, String, Vec<InlineNodeItem<'c, 'g, N>>) {
  let mut boxes = Vec::new();
  let mut text_spans = Vec::new();

  let (mut layout, text) = global.font_context.tree_builder(style.into(), |builder| {
    let mut idx = 0;
    let mut index_pos = 0;

    for item in items {
      match item {
        InlineItem::Text { text, context } => {
          let transformed = apply_text_transform(&text, context.style.text_transform);
          let collapsed =
            apply_white_space_collapse(&transformed, style.parent.white_space_collapse());

          builder.push_style_span((&context.style.to_sized_font_style(context)).into());
          builder.push_text(&collapsed);
          builder.pop_style_span();

          index_pos += collapsed.len();

          text_spans.push((idx, collapsed.into_owned()));
        }
        InlineItem::Node(item) => {
          let size = item.node.measure(
            item.context,
            available_space,
            Size::NONE,
            &taffy::Style::default(),
          );

          let inline_box = InlineBox {
            index: index_pos,
            id: idx,
            width: size.width,
            height: size.height,
          };

          boxes.push(item);

          builder.push_inline_box(inline_box);

          idx += 1;
        }
      }
    }
  });

  break_lines(&mut layout, max_width, max_height);

  if measure_only {
    return (layout, text, boxes);
  }

  let text_wrap_style = style
    .parent
    .text_wrap_style
    .unwrap_or(style.parent.text_wrap.style);
  let line_count = layout.lines().count();

  if text_wrap_style == TextWrapStyle::Balance {
    make_balanced_text(&mut layout, max_width, line_count);
  }

  if text_wrap_style == TextWrapStyle::Pretty {
    make_pretty_text(&mut layout, max_width);
  }

  layout.align(
    Some(max_width),
    style.parent.text_align.into(),
    Default::default(),
  );

  (layout, text, boxes)
}

pub(crate) fn create_inline_constraint(
  context: &RenderContext,
  available_space: Size<AvailableSpace>,
  known_dimensions: Size<Option<f32>>,
) -> (f32, Option<MaxHeight>) {
  let width_constraint = known_dimensions
    .width
    .or(match available_space.width {
      AvailableSpace::MinContent => Some(0.0),
      AvailableSpace::MaxContent => None,
      AvailableSpace::Definite(width) => Some(width),
    })
    .unwrap_or(f32::MAX);

  // applies a maximum height to reduce unnecessary calculation.
  let max_height = match (
    context.sizing.viewport.height,
    context.style.text_wrap_mode_and_line_clamp().1,
  ) {
    (Some(height), Some(line_clamp)) => {
      Some(MaxHeight::HeightAndLines(height as f32, line_clamp.count))
    }
    (Some(height), None) => Some(MaxHeight::Absolute(height as f32)),
    (None, Some(line_clamp)) => Some(MaxHeight::Lines(line_clamp.count)),
    (None, None) => None,
  };

  (width_constraint, max_height)
}

pub(crate) fn break_lines(
  layout: &mut InlineLayout,
  max_width: f32,
  max_height: Option<MaxHeight>,
) {
  let Some(max_height) = max_height else {
    return layout.break_all_lines(Some(max_width));
  };

  match max_height {
    MaxHeight::Lines(lines) => {
      let mut breaker = layout.break_lines();

      for _ in 0..lines {
        if breaker.break_next(max_width).is_none() {
          // no more lines to break
          break;
        };
      }

      breaker.finish();
    }
    MaxHeight::Absolute(max_height) => {
      let mut total_height = 0.0;
      let mut breaker = layout.break_lines();

      while total_height < max_height {
        let Some((_, height)) = breaker.break_next(max_width) else {
          // no more lines to break
          break;
        };

        total_height += height;
      }

      // if its over the max height after last break, revert the break
      if total_height > max_height {
        breaker.revert();
      }

      breaker.finish();
    }
    MaxHeight::HeightAndLines(max_height, max_lines) => {
      let mut total_height = 0.0;
      let mut line_count = 0;
      let mut breaker = layout.break_lines();

      while total_height < max_height {
        if line_count >= max_lines {
          break;
        }

        let Some((_, height)) = breaker.break_next(max_width) else {
          // no more lines to break
          break;
        };

        line_count += 1;
        total_height += height;
      }

      if total_height > max_height {
        breaker.revert();
      }

      breaker.finish();
    }
  }
}
