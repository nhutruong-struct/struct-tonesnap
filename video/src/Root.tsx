import React from "react";
import { AbsoluteFill, Audio, Composition, Sequence, staticFile } from "remotion";
import { FPS } from "./theme";
import { Background } from "./components/ui";
import { Hook, Intro } from "./scenes/Opening";
import { ApplyStep, OptionsStep, SelectStep, TriggerStep } from "./scenes/Steps";
import { FEATURE_FRAMES, Features } from "./scenes/Features";
import { Outro, Setup } from "./scenes/Closing";

const s = (seconds: number) => Math.round(seconds * FPS);

/** Scene lengths in seconds, in playing order. */
const SCENES: { name: string; seconds: number; node: React.ReactNode }[] = [
  { name: "Hook", seconds: 5, node: <Hook /> },
  { name: "Intro", seconds: 5, node: <Intro /> },
  { name: "Select", seconds: 9, node: <SelectStep /> },
  { name: "Trigger", seconds: 12, node: <TriggerStep /> },
  { name: "Options", seconds: 9, node: <OptionsStep /> },
  { name: "Apply", seconds: 9, node: <ApplyStep /> },
  { name: "Features", seconds: (FEATURE_FRAMES * 4) / FPS, node: <Features /> },
  { name: "Setup", seconds: 8, node: <Setup /> },
  { name: "Outro", seconds: 6, node: <Outro /> },
];

export const TOTAL = SCENES.reduce((sum, sc) => sum + s(sc.seconds), 0);

/** Sound effects by global frame, each on the on-screen event it goes with (see the scene files). */
const SFX: { name: string; frames: number[]; volume?: number }[] = [
  {
    name: "swish",
    frames: [6, 50, 84, 190, 212, 300, 412, 720, 972, 1062, 1470, 1590, 1710, 1830, 1950, 2220],
    volume: 0.35,
  },
  { name: "pop", frames: [154, 472, 1296, 1964, 1994, 2024, 2192] },
  { name: "click", frames: [614] },
  { name: "whoosh", frames: [620, 930, 1292], volume: 0.4 },
  { name: "thock", frames: [758, 1270] },
  { name: "tick", frames: [970, 1060], volume: 0.5 },
  { name: "chime", frames: [833, 1312, 2246], volume: 0.45 },
];

export const ToneSnapIntro: React.FC = () => {
  let from = 0;
  return (
    <AbsoluteFill>
      <Background />
      <Audio src={staticFile("music.wav")} volume={0.5} />
      {SFX.flatMap((fx) =>
        fx.frames.map((f) => (
          <Sequence key={`${fx.name}-${f}`} from={f} layout="none">
            <Audio src={staticFile(`sfx/${fx.name}.wav`)} volume={fx.volume ?? 0.55} />
          </Sequence>
        )),
      )}
      {SCENES.map((sc) => {
        const start = from;
        from += s(sc.seconds);
        return (
          <Sequence key={sc.name} name={sc.name} from={start} durationInFrames={s(sc.seconds)}>
            {sc.node}
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};

export const Root: React.FC = () => (
  <Composition
    id="ToneSnapIntro"
    component={ToneSnapIntro}
    durationInFrames={TOTAL}
    fps={FPS}
    width={1920}
    height={1080}
  />
);
