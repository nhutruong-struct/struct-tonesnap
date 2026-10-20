import { useId, type ReactNode } from "react";

/** Line icons on a 24 × 24 grid; they take the colour of the text around them. */
const PATHS = {
  close: <path d="M6 6l12 12M18 6L6 18" />,
  settings: (
    <>
      <path d="M10.03 4.66L10.66 2.49H13.34L13.97 4.66L15.8 5.42L17.78 4.33L19.67 6.22L18.58 8.2L19.34 10.03L21.51 10.66V13.34L19.34 13.97L18.58 15.8L19.67 17.78L17.78 19.67L15.8 18.58L13.97 19.34L13.34 21.51H10.66L10.03 19.34L8.2 18.58L6.22 19.67L4.33 17.78L5.42 15.8L4.66 13.97L2.49 13.34V10.66L4.66 10.03L5.42 8.2L4.33 6.22L6.22 4.33L8.2 5.42Z" />
      <circle cx="12" cy="12" r="3" />
    </>
  ),
  sliders: (
    <>
      <path d="M4 7h9M17 7h3M4 17h3M11 17h9" />
      <circle cx="15" cy="7" r="2" />
      <circle cx="9" cy="17" r="2" />
    </>
  ),
  sparkles: (
    <path d="M12 3c.6 4.6 2.4 6.4 7 7-4.6.6-6.4 2.4-7 7-.6-4.6-2.4-6.4-7-7 4.6-.6 6.4-2.4 7-7zM19 16v4M17 18h4" />
  ),
  keyboard: (
    <>
      <rect x="3" y="6" width="18" height="12" rx="2.5" />
      <path d="M7 10h.01M11 10h.01M15 10h.01M8 14h8" />
    </>
  ),
  type: <path d="M5 7V5h14v2M12 5v14M9 19h6" />,
  layers: <path d="M12 4l8 4-8 4-8-4 8-4zM4 12l8 4 8-4M4 16l8 4 8-4" />,
  shield: (
    <>
      <path d="M12 3l7 3v5c0 4.5-3 8-7 10-4-2-7-5.5-7-10V6l7-3z" />
      <path d="M9 12l2 2 4-4" />
    </>
  ),
  lock: (
    <>
      <rect x="5" y="11" width="14" height="9" rx="2" />
      <path d="M8 11V8a4 4 0 018 0v3" />
    </>
  ),
  refresh: <path d="M20 11a8 8 0 10-2.3 5.7M20 4v7h-7" />,
  warning: <path d="M12 4l9 16H3L12 4zM12 10v4M12 17h.01" />,
  info: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 11v5M12 8h.01" />
    </>
  ),
} satisfies Record<string, ReactNode>;

export type IconName = keyof typeof PATHS;

export function Icon({ name, size = 16 }: { name: IconName; size?: number }) {
  return (
    <svg
      className="icon-svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {PATHS[name]}
    </svg>
  );
}

/** The ToneSnap app icon (`src-tauri/icons/source/tonesnap.svg`). */
export function Logo({ size = 24 }: { size?: number }) {
  const gradient = useId();
  return (
    <svg className="icon-svg" width={size} height={size} viewBox="32 32 960 960" aria-hidden="true">
      <defs>
        <linearGradient id={gradient} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#3B7BFF" />
          <stop offset="1" stopColor="#7A5CFA" />
        </linearGradient>
      </defs>
      <rect x="32" y="32" width="960" height="960" rx="224" fill={`url(#${gradient})`} />
      <rect x="252" y="262" width="520" height="132" rx="66" fill="#FFFFFF" />
      <rect x="446" y="262" width="132" height="500" rx="66" fill="#FFFFFF" />
      <path d="M742 560 Q756 646 842 660 Q756 674 742 760 Q728 674 642 660 Q728 646 742 560 Z" fill="#FFD45C" />
    </svg>
  );
}
