---
"takumi": minor
---

**Migrate to pure Node struct without generic support**

Before:

```rust
let mut node = NodeKind::Container(ContainerNode {
  children: Some(Box::from([
    NodeKind::Text(TextNode {
      text: "Hello, world!".to_string(),
      style: None,
      tw: None,
      preset: None,
      tag_name: None,
      class_name: None,
      id: None,
    }),
  ])),
  preset: None,
  style: None,
  tw: None,
  tag_name: None,
  class_name: None,
  id: Some("root".to_string()),
});
```

After:

```rust
let node = Node::container([Node::text("Hello, world!")]).with_id("root");
```
