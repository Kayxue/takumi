import { readdir, readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { compile } from "tailwindcss";

const require = createRequire(import.meta.url);

export async function compileTailwindStylesheet(directory: string) {
  const tailwindInput = await readFile(join(directory, "input.css"), "utf8");
  const candidates = await collectTailwindCandidates(directory);
  const tailwindCompiler = await compile(tailwindInput, {
    base: directory,
    loadStylesheet: async (id, base) => {
      const path = resolveStylesheetPath(id, base);
      return {
        path,
        base: dirname(path),
        content: await readFile(path, "utf8"),
      };
    },
  });

  return tailwindCompiler.build(candidates);
}

function resolveStylesheetPath(id: string, base: string) {
  if (id === "tailwindcss") {
    return require.resolve("tailwindcss/index.css");
  }

  if (id.startsWith(".")) {
    return resolve(base, id);
  }

  if (isAbsolute(id)) {
    return id;
  }

  return require.resolve(id);
}

async function collectTailwindCandidates(directory: string) {
  const entries = await readdir(directory, { withFileTypes: true });
  const tokens = new Set<string>();

  for (const entry of entries) {
    if (entry.isDirectory()) continue;
    if (!entry.name.endsWith(".tsx") && !entry.name.endsWith(".ts")) continue;

    const source = await readFile(join(directory, entry.name), "utf8");

    for (const match of source.matchAll(/className\s*=\s*"([^"]+)"/g)) {
      addClasses(tokens, match[1]);
    }

    for (const match of source.matchAll(/className\s*=\s*\{`([^`]+)`\}/g)) {
      addClasses(tokens, match[1]);
    }
  }

  return [...tokens];
}

function addClasses(tokens: Set<string>, value: string | undefined) {
  if (!value) return;

  for (const token of value.split(/\s+/)) {
    if (token.length > 0) {
      tokens.add(token);
    }
  }
}
