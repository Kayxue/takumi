import { createGenerator, presetWind4 } from "unocss";
import { collectClassCandidates } from "./class-candidates";

const uno = await createGenerator({
  presets: [presetWind4()],
});

export async function compileUnoStylesheet(directory: string) {
  const candidates = await collectClassCandidates(directory);
  const { css } = await uno.generate(candidates.join(" "));

  return css;
}
