// Synthesizes the background music and sound effects as WAV files, with no samples and no
// dependencies, so the audio is original and free to use. Run: node gen-audio.mjs
import fs from "node:fs";

const SR = 44100;
const TAU = Math.PI * 2;

const makeBuf = (seconds) => ({
  L: new Float32Array(Math.ceil(seconds * SR)),
  R: new Float32Array(Math.ceil(seconds * SR)),
});

/** Mixes a mono function of time (seconds since the note started) into the buffer. */
function voice(buf, start, dur, pan, fn) {
  const a = Math.floor(start * SR);
  const n = Math.floor(dur * SR);
  const gl = Math.cos(((pan + 1) * Math.PI) / 4);
  const gr = Math.sin(((pan + 1) * Math.PI) / 4);
  for (let i = 0; i < n && a + i < buf.L.length; i++) {
    const v = fn(i / SR);
    buf.L[a + i] += v * gl;
    buf.R[a + i] += v * gr;
  }
}

const decay = (t, tau) => Math.exp(-t / tau);
const attack = (t, a) => Math.min(1, t / a);

function writeWav(path, buf, peak = 0.85) {
  let max = 1e-9;
  for (let i = 0; i < buf.L.length; i++) max = Math.max(max, Math.abs(buf.L[i]), Math.abs(buf.R[i]));
  const g = Math.min(1, peak / max);
  const out = Buffer.alloc(44 + buf.L.length * 4);
  out.write("RIFF", 0);
  out.writeUInt32LE(36 + buf.L.length * 4, 4);
  out.write("WAVEfmt ", 8);
  out.writeUInt32LE(16, 16);
  out.writeUInt16LE(1, 20);
  out.writeUInt16LE(2, 22);
  out.writeUInt32LE(SR, 24);
  out.writeUInt32LE(SR * 4, 28);
  out.writeUInt16LE(4, 32);
  out.writeUInt16LE(16, 34);
  out.write("data", 36);
  out.writeUInt32LE(buf.L.length * 4, 40);
  for (let i = 0; i < buf.L.length; i++) {
    out.writeInt16LE(Math.round(Math.max(-1, Math.min(1, buf.L[i] * g)) * 32767), 44 + i * 4);
    out.writeInt16LE(Math.round(Math.max(-1, Math.min(1, buf.R[i] * g)) * 32767), 46 + i * 4);
  }
  fs.writeFileSync(path, out);
  return { max, gain: g };
}

/** A small feedback-comb reverb mixed in under the dry signal. */
function reverb(buf, wet = 0.22) {
  const delays = [0.0297, 0.0371, 0.0411, 0.0437, 0.0533].map((d) => Math.floor(d * SR));
  for (const ch of ["L", "R"]) {
    const dry = buf[ch];
    const acc = new Float32Array(dry.length);
    delays.forEach((d, k) => {
      const dd = d + (ch === "R" ? 13 * (k + 1) : 0);
      const line = new Float32Array(dry.length);
      for (let i = 0; i < dry.length; i++) {
        line[i] = dry[i] + (i >= dd ? line[i - dd] * 0.8 : 0);
        acc[i] += line[i] / delays.length;
      }
    });
    for (let i = 0; i < dry.length; i++) dry[i] += acc[i] * wet;
  }
}

fs.mkdirSync("public/sfx", { recursive: true });

// ---------------------------------------------------------------- music
const BPM = 96;
const BEAT = 60 / BPM;
const BAR = BEAT * 4;
const DURATION = 79;
const bars = Math.ceil(DURATION / BAR) + 1;
const music = makeBuf(bars * BAR + 3);

// Am, F, C, G; the last bar resolves to C.
const CHORDS = [
  { root: 110.0, pad: [220.0, 261.63, 329.63], arp: [440.0, 523.25, 659.25] },
  { root: 87.31, pad: [174.61, 220.0, 261.63], arp: [349.23, 440.0, 523.25] },
  { root: 130.81, pad: [261.63, 329.63, 392.0], arp: [523.25, 659.25, 783.99] },
  { root: 98.0, pad: [246.94, 293.66, 392.0], arp: [392.0, 493.88, 587.33] },
];
const ARP_PATTERN = [0, 1, 2, 1, 0, 2, 1, 2];

let seed = 7;
const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296) * 2 - 1;

for (let bar = 0; bar < bars; bar++) {
  const last = (bar + 1) * BAR >= DURATION;
  const ch = last ? CHORDS[2] : CHORDS[bar % 4];
  const t0 = bar * BAR;

  // Pad: soft detuned triangle-ish tones that swell in and out of each bar.
  ch.pad.forEach((f, k) => {
    for (const det of [0.997, 1.003]) {
      voice(music, t0, BAR + 0.8, (k - 1) * 0.5, (t) => {
        const env = attack(t, 0.7) * Math.min(1, Math.max(0, (BAR + 0.8 - t) / 0.8));
        return (Math.sin(TAU * f * det * t) + 0.25 * Math.sin(TAU * 2 * f * det * t)) * 0.035 * env;
      });
    }
  });

  // Arpeggio plucks from bar 2 on, eighth notes, alternating sides.
  if (bar >= 2) {
    ARP_PATTERN.forEach((idx, step) => {
      const f = ch.arp[idx] * (step === 7 && bar % 2 ? 2 : 1);
      const accent = step % 4 === 0 ? 1 : 0.7;
      voice(music, t0 + step * (BEAT / 2), 1.1, step % 2 ? 0.45 : -0.45, (t) => {
        const env = attack(t, 0.004) * decay(t, 0.22);
        return (Math.sin(TAU * f * t) + 0.35 * Math.sin(TAU * 2 * f * t) * decay(t, 0.1)) * 0.075 * accent * env;
      });
    });
  }

  // Bass from bar 4 on, on beats 1 and 3.
  if (bar >= 4) {
    for (const beat of [0, 2]) {
      voice(
        music,
        t0 + beat * BEAT,
        1.2,
        0,
        (t) => Math.sin(TAU * ch.root * t) * 0.2 * attack(t, 0.01) * decay(t, 0.55),
      );
    }
  }

  // Light drums from bar 8 until the last two bars.
  if (bar >= 8 && bar < bars - 3) {
    for (let beat = 0; beat < 4; beat++) {
      voice(music, t0 + beat * BEAT, 0.3, 0, (t) => {
        const f = 45 + 80 * Math.exp(-t / 0.035);
        return Math.sin(TAU * f * t) * 0.3 * decay(t, 0.09);
      });
      voice(music, t0 + beat * BEAT + BEAT / 2, 0.06, 0.3, (t) => {
        // Differentiated noise is a crude high-pass for the hat.
        return (rnd() - rnd() * 0.5) * 0.045 * decay(t, 0.012);
      });
    }
  }
}
reverb(music);
// Fade in over 1.5 s and out over the last 3.5 s.
for (let i = 0; i < music.L.length; i++) {
  const t = i / SR;
  const g = Math.min(1, t / 1.5) * Math.min(1, Math.max(0, (DURATION - t) / 3.5));
  music.L[i] = Math.tanh(music.L[i] * 1.4) * g;
  music.R[i] = Math.tanh(music.R[i] * 1.4) * g;
}
// Cut to the video length.
const musicOut = { L: music.L.slice(0, DURATION * SR), R: music.R.slice(0, DURATION * SR) };
console.log("music", writeWav("public/music.wav", musicOut, 0.8));

// ---------------------------------------------------------------- sound effects
const sfx = {};
const make = (name, seconds, build) => {
  const b = makeBuf(seconds);
  build(b);
  sfx[name] = b;
};

make("pop", 0.3, (b) =>
  voice(
    b,
    0,
    0.3,
    0,
    (t) => Math.sin(TAU * (520 + 700 * (1 - Math.exp(-t / 0.03))) * t) * 0.6 * attack(t, 0.002) * decay(t, 0.07),
  ),
);
make("click", 0.15, (b) => {
  voice(b, 0, 0.1, 0, (t) => rnd() * 0.5 * decay(t, 0.004) + Math.sin(TAU * 1700 * t) * 0.4 * decay(t, 0.012));
});
make("thock", 0.2, (b) => {
  voice(b, 0, 0.2, 0, (t) => Math.sin(TAU * 190 * t) * 0.6 * decay(t, 0.035) + rnd() * 0.25 * decay(t, 0.004));
});
make("tick", 0.1, (b) =>
  voice(b, 0, 0.08, 0, (t) => Math.sin(TAU * 2300 * t) * 0.35 * attack(t, 0.001) * decay(t, 0.012)),
);
make("chime", 1.4, (b) => {
  [784, 1174.66].forEach((f, k) =>
    voice(
      b,
      k * 0.09,
      1.3,
      k ? 0.3 : -0.3,
      (t) => (Math.sin(TAU * f * t) + 0.3 * Math.sin(TAU * 2 * f * t)) * 0.3 * attack(t, 0.004) * decay(t, 0.35),
    ),
  );
});
const whoosh = (seconds, peak) => (b) => {
  let lp = 0;
  voice(b, 0, seconds, 0, (t) => {
    const x = t / seconds;
    const cutoff = 0.02 + 0.22 * Math.sin(Math.PI * x) ** 2;
    lp += (rnd() - lp) * cutoff;
    return lp * peak * Math.sin(Math.PI * x) ** 1.5;
  });
};
make("whoosh", 0.5, whoosh(0.5, 2.2));
make("swish", 0.3, whoosh(0.3, 1.4));

for (const [name, b] of Object.entries(sfx)) {
  if (name === "chime") reverb(b, 0.3);
  console.log(name, writeWav(`public/sfx/${name}.wav`, b, 0.8));
}
