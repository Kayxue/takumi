import { file, write } from "bun";

const generated = await file("pkg/takumi_wasm.d.ts").text();
const custom = await file("src/dts-header.d.ts").text();

await write("pkg/takumi_wasm.d.ts", `${custom.trim()}\n\n${generated}`);
