use std::mem::take;

use parley::InlineBox;
use taffy::{AvailableSpace, NodeId, Size, TaffyTree};

use crate::{
  layout::{
    inline::{InlineContentKind, InlineItem, InlineLayout, break_lines, create_inline_constraint},
    node::Node,
    style::{Display, InheritedStyle},
  },
  rendering::{MaxHeight, RenderContext},
};

pub(crate) struct NodeTree<'g, N: Node<N>> {
  pub(crate) context: RenderContext<'g>,
  pub(crate) node: Option<N>,
  children: Option<Vec<NodeTree<'g, N>>>,
}

impl<'g, N: Node<N>> NodeTree<'g, N> {
  pub fn is_inline(&self) -> bool {
    self.context.style.display == Display::Inline
  }

  pub fn should_construct_inline_layout(&self) -> bool {
    self.context.style.display == Display::Block
      && self
        .children
        .as_ref()
        .is_some_and(|children| children.iter().any(NodeTree::is_inline))
  }

  pub fn from_node(parent_context: &RenderContext<'g>, node: N) -> Self {
    let mut tree = Self::from_node_impl(parent_context, node);

    // https://www.w3.org/TR/css-display-3/#root
    // The root element’s display type is always blockified.
    if tree.is_inline() {
      tree.context.style.display.to_block();
    }

    tree
  }

  fn from_node_impl(parent_context: &RenderContext<'g>, mut node: N) -> Self {
    let style = node.take_style().inherit(&parent_context.style);

    let font_size = style
      .font_size
      .map(|font_size| font_size.resolve_to_px(parent_context, parent_context.font_size))
      .unwrap_or(parent_context.font_size);

    let current_color = style.color.resolve(parent_context.current_color);

    let mut context = RenderContext {
      style,
      font_size,
      current_color,
      ..*parent_context
    };

    let children = node.take_children().map(|children| {
      children
        .into_iter()
        .map(|child| Self::from_node_impl(&context, child))
        .collect::<Vec<_>>()
    });

    let Some(mut children) = children else {
      return Self {
        context,
        node: Some(node),
        children: None,
      };
    };

    if context.style.display.should_blockify_children() {
      for child in &mut children {
        child.context.style.display.to_block();
      }

      return Self {
        context,
        node: Some(node),
        children: Some(children),
      };
    }

    let has_inline = children.iter().any(NodeTree::is_inline);
    let has_block = children.iter().any(|child| !child.is_inline());
    let needs_anonymous_boxes = context.style.display == Display::Block && has_inline && has_block;

    if !needs_anonymous_boxes {
      return Self {
        context,
        node: Some(node),
        children: Some(children),
      };
    }

    context.style.display = context.style.display.as_block();

    let mut final_children = Vec::new();
    let mut inline_group = Vec::new();

    // Anonymous block box style.
    let anonymous_box_style = InheritedStyle {
      display: Display::Block,
      ..InheritedStyle::default()
    };

    for item in children {
      if !item.is_inline() {
        if !inline_group.is_empty() {
          final_children.push(NodeTree {
            context: RenderContext {
              style: anonymous_box_style.clone(),
              ..context
            },
            children: Some(take(&mut inline_group)),
            node: None,
          });
        }

        final_children.push(item);
        continue;
      }

      inline_group.push(item);
    }

    if !inline_group.is_empty() {
      final_children.push(NodeTree {
        context: RenderContext {
          style: anonymous_box_style,
          ..context
        },
        children: Some(inline_group),
        node: None,
      });
    }

    Self {
      context,
      node: Some(node),
      children: Some(final_children),
    }
  }

  pub(crate) fn insert_into_taffy(mut self, tree: &mut TaffyTree<NodeTree<'g, N>>) -> NodeId {
    if self.context.style.display == Display::Inline {
      unreachable!("Inline nodes should be wrapped in anonymous block boxes");
    }

    if self.should_construct_inline_layout() {
      return tree
        .new_leaf_with_context(self.context.style.to_taffy_style(&self.context), self)
        .unwrap();
    }

    let children = self.children.take();

    let node_id = tree
      .new_leaf_with_context(self.context.style.to_taffy_style(&self.context), self)
      .unwrap();

    if let Some(children) = children {
      let children_ids = children
        .into_iter()
        .map(|child| child.insert_into_taffy(tree))
        .collect::<Vec<_>>();

      tree.set_children(node_id, &children_ids).unwrap();
    }

    node_id
  }

  pub(crate) fn measure(
    &self,
    available_space: Size<AvailableSpace>,
    known_dimensions: Size<Option<f32>>,
  ) -> Size<f32> {
    if self.should_construct_inline_layout() {
      let (max_width, max_height) =
        create_inline_constraint(&self.context, available_space, known_dimensions);

      let font_style = self.context.style.to_sized_font_style(&self.context);

      let mut boxes = Vec::new();

      let (mut layout, _) =
        self
          .context
          .global
          .font_context
          .tree_builder((&font_style).into(), |builder| {
            let mut idx = 0;
            let mut index_pos = 0;

            for (item, context) in self.inline_items_iter() {
              match item {
                InlineItem::Text(text) => {
                  builder.push_style_span((&context.style.to_sized_font_style(context)).into());
                  builder.push_text(&text);
                  builder.pop_style_span();

                  index_pos += text.len();
                }
                InlineItem::Node(node) => {
                  let size = node.measure(context, available_space, Size::NONE);

                  boxes.push(size);

                  builder.push_inline_box(InlineBox {
                    index: index_pos,
                    id: idx,
                    width: size.width,
                    height: size.height,
                  });

                  idx += 1;
                }
              }
            }
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

      return taffy::Size {
        width: max_run_width.ceil().min(max_width),
        height: total_height.ceil(),
      };
    }

    if self.context.style.display == Display::Inline {
      unreachable!("Inline nodes should be wrapped in anonymous block boxes");
    }

    let Some(node) = &self.node else {
      return Size::zero();
    };

    node.measure(&self.context, available_space, known_dimensions)
  }

  pub(crate) fn create_inline_layout(&self, size: Size<f32>) -> (InlineLayout, String, Vec<&N>) {
    let font_style = self.context.style.to_sized_font_style(&self.context);
    let mut boxes = Vec::new();

    let (mut layout, text) =
      self
        .context
        .global
        .font_context
        .tree_builder((&font_style).into(), |builder| {
          let mut index_pos = 0;

          for (item, context) in self.inline_items_iter() {
            match item {
              InlineItem::Text(text) => {
                builder.push_style_span((&context.style.to_sized_font_style(context)).into());
                builder.push_text(&text);
                builder.pop_style_span();

                index_pos += text.len();
              }
              InlineItem::Node(node) => {
                let size = node.measure(
                  context,
                  Size {
                    width: AvailableSpace::Definite(size.width),
                    height: AvailableSpace::Definite(size.height),
                  },
                  Size::NONE,
                );

                builder.push_inline_box(InlineBox {
                  index: index_pos,
                  id: boxes.len() as u64,
                  width: size.width,
                  height: size.height,
                });

                boxes.push(node);
              }
            }
          }
        });

    let max_height = match font_style.parent.line_clamp.as_ref() {
      Some(clamp) => Some(MaxHeight::Both(size.height, clamp.count)),
      None => Some(MaxHeight::Absolute(size.height)),
    };

    break_lines(&mut layout, size.width, max_height);

    layout.align(
      Some(size.width),
      self.context.style.text_align.into(),
      Default::default(),
    );

    (layout, text, boxes)
  }

  fn inline_items_iter(&self) -> InlineItemIterator<'_, N> {
    if self.context.style.display != Display::Block {
      panic!("Root node must be display block");
    }

    InlineItemIterator {
      stack: vec![(self, 0)], // (node, depth)
      current_node_content: None,
    }
  }
}

/// Iterator for traversing inline items in document order
pub(crate) struct InlineItemIterator<'g, N: Node<N>> {
  stack: Vec<(&'g NodeTree<'g, N>, usize)>, // (node, depth)
  current_node_content: Option<(InlineItem<'g, N>, &'g RenderContext<'g>)>,
}

impl<'g, N: Node<N>> Iterator for InlineItemIterator<'g, N> {
  type Item = (InlineItem<'g, N>, &'g RenderContext<'g>);

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      // If we have current node content to return, return it
      if let Some(content) = self.current_node_content.take() {
        return Some(content);
      }

      // Get the next node from the stack
      let (node, depth) = self.stack.pop()?;

      // Validate display type for non-root nodes
      if depth > 0 && node.context.style.display != Display::Inline {
        panic!("Non-root nodes must be display inline");
      }

      // Push children onto stack in reverse order (so they process in forward order)
      if let Some(children) = &node.children {
        for child in children.iter().rev() {
          self.stack.push((child, depth + 1));
        }
      }

      // Prepare the current node's content
      if let Some(inline_content) = node
        .node
        .as_ref()
        .and_then(|n| n.inline_content(&node.context))
      {
        match inline_content {
          InlineContentKind::Box => {
            if let Some(n) = &node.node {
              self.current_node_content = Some((InlineItem::Node(n), &node.context));
            }
          }
          InlineContentKind::Text(text) => {
            self.current_node_content = Some((InlineItem::Text(text.into()), &node.context));
          }
        }
      }
    }
  }
}
