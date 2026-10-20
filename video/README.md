# ToneSnap intro video

Motion-graphics intro for ToneSnap, built with [Remotion](https://www.remotion.dev/) (React). 1920×1080, 30 fps,
about 79 s, English captions, original music and sound effects synthesized by `gen-audio.mjs` (no samples, so
nothing to license). The rendered video lives in `../site/media/` and is shown on the
[website](https://nhutruong-struct.github.io/struct-tonesnap/#demo) and in the main README.

It is a separate project: its own `package.json` and lockfile, not part of the app's build, and nothing in
`src-tauri/` or `src/` depends on it. Run the commands below from this folder after `pnpm install`.

| Command | Does |
|---|---|
| `pnpm studio` | Opens the preview and timeline editor |
| `pnpm render` | Regenerates the audio, then writes `../site/media/tonesnap-intro.mp4` |
| `pnpm poster` | Writes `../site/media/tonesnap-intro-poster.jpg` (frame 870, the popup with three tones) |
| `pnpm audio` | Regenerates `public/music.wav` and `public/sfx/*.wav` (git-ignored, about 14 MB) |
| `pnpm typecheck` | Type-checks the scenes |
| `node stills.mjs 500 870` | Writes half-size PNGs of those frames to `out/` for a quick look |

Scenes (`src/Root.tsx`): hook, intro, 1 select, 2 trigger, language and mode, 3 apply, four features, setup,
outro. Texts for the demo are in `src/data.ts`; colors follow the app's tokens in `src/theme.ts`; music volume
and the position of each sound effect are in `src/Root.tsx`, tempo and chords at the top of `gen-audio.mjs`.

When the popup or the flow changes in a visible way, update the scenes, run `pnpm render` and `pnpm poster`, and
commit the new files in `../site/media/`. The first render downloads Chrome Headless Shell (about 120 MB).

Remotion is free for individuals and small companies; check its [license](https://www.remotion.dev/license)
before using this project in a larger company.
