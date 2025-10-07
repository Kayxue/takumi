//! Text node implementation for the takumi layout system.
//!
//! This module contains the TextNode struct which is used to render
//! text content with configurable font properties and styling.

use serde::{Deserialize, Serialize};
use taffy::{AvailableSpace, Layout, Size};

use crate::{
  layout::{
    inline::{InlineContentKind, break_lines, create_inline_constraint},
    node::Node,
    style::Style,
  },
  rendering::{
    Canvas, MaxHeight, RenderContext, apply_text_transform, inline_drawing::draw_inline_layout,
  },
};

/// A node that renders text content.
///
/// Text nodes display text with configurable font properties,
/// alignment, and styling options.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextNode {
  /// The styling properties for this text node
  pub style: Option<Style>,
  /// The text content to be rendered
  pub text: String,
}

impl<Nodes: Node<Nodes>> Node<Nodes> for TextNode {
  fn take_style(&mut self) -> Style {
    self.style.take().unwrap_or_default()
  }

  fn inline_content(&self, context: &RenderContext) -> Option<InlineContentKind> {
    Some(InlineContentKind::Text(
      apply_text_transform(&self.text, context.style.text_transform).to_string(),
    ))
  }

  fn draw_content(&self, context: &RenderContext, canvas: &Canvas, layout: Layout) {
    let font_style = context.style.to_sized_font_style(context);

    let (mut inline_layout, _) =
      context
        .global
        .font_context
        .tree_builder((&font_style).into(), |builder| {
          builder.push_text(&apply_text_transform(
            &self.text,
            context.style.text_transform,
          ));
        });

    let size = layout.content_box_size();

    let max_height = match font_style.parent.line_clamp.as_ref() {
      Some(clamp) => Some(MaxHeight::Both(size.height, clamp.count)),
      None => Some(MaxHeight::Absolute(size.height)),
    };

    break_lines(&mut inline_layout, size.width, max_height);

    inline_layout.align(
      Some(size.width),
      context.style.text_align.into(),
      Default::default(),
    );

    draw_inline_layout(context, canvas, layout, inline_layout, &font_style);
  }

  fn measure(
    &self,
    context: &RenderContext,
    available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
  ) -> Size<f32> {
    let (max_width, max_height) =
      create_inline_constraint(context, available_space, known_dimensions);

    let font_style = context.style.to_sized_font_style(context);

    let (mut layout, _) =
      context
        .global
        .font_context
        .tree_builder((&font_style).into(), |builder| {
          builder.push_text(&apply_text_transform(
            &self.text,
            context.style.text_transform,
          ));
        });

    break_lines(&mut layout, max_width, max_height);

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

    taffy::Size {
      width: max_run_width.ceil().min(max_width),
      height: total_height.ceil(),
    }
  }
}
