import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { compile } from "tailwindcss";
import { collectClassCandidates } from "./class-candidates";

const require = createRequire(import.meta.url);

export async function compileTailwindStylesheet(directory: string) {
  const tailwindInput = await readFile(join(directory, "input.css"), "utf8");
  const candidates = await collectClassCandidates(directory);
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
