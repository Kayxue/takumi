import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";

export async function collectClassCandidates(directory: string) {
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
