# Tailwind CSS Stylesheet Example

This example compiles Tailwind CSS in-process with `compile()` and passes the resulting CSS into Takumi's `stylesheets` render option.

## Run

Before running the example, build the native package once from the workspace root.

```bash
bun --filter '*' run build
```

Then render the example image.

```bash
cd example/tailwindcss
bun run render
```

This writes `output/tailwind-stylesheets.png`.
