import { mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { Renderer } from "@takumi-rs/core";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import { write } from "bun";
import { TailwindCard } from "./card";
import { compileTailwindStylesheet } from "./tailwind-compile";

const width = 1200;
const height = 630;

const currentFile = fileURLToPath(import.meta.url);
const currentDir = dirname(currentFile);
const exampleDir = dirname(currentDir);
const outputDir = join(exampleDir, "output");
const stylesheet = await compileTailwindStylesheet(currentDir);

await write(join(outputDir, "styles.generated.css"), stylesheet);

const renderer = new Renderer();

const { node } = await fromJsx(<TailwindCard />);

const image = await renderer.render(node, {
  width,
  height,
  stylesheets: [stylesheet],
});

await mkdir(outputDir, { recursive: true });
await write(join(outputDir, "tailwind-stylesheets.png"), image);
