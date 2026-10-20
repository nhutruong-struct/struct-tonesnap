import { bundle } from "@remotion/bundler";
import { renderStill, selectComposition } from "@remotion/renderer";
import path from "node:path";

const frames = process.argv.slice(2).map(Number);
const serveUrl = await bundle({ entryPoint: path.resolve("src/index.ts") });
const composition = await selectComposition({ serveUrl, id: "ToneSnapIntro" });
for (const frame of frames) {
  await renderStill({ composition, serveUrl, frame, output: `out/still-${frame}.png`, scale: 0.5 });
}
console.log("done", frames.join(","));
