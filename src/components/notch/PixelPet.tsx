/**
 * Pixel Pet — the 8-bit character from the original Vibe Island.
 * Each state has a unique pixel art sprite rendered with crisp SVG rects.
 * Colors come from CSS variables --vi-idle, --vi-work, --vi-alert, --vi-question.
 */

import type { SessionStatus } from "../../types";

interface Props {
  status: SessionStatus;
  size?: number;
}

const STATE_COLORS: Record<string, { body: string; bright: string }> = {
  idle: { body: "var(--vi-idle, #22c55e)", bright: "var(--vi-idle-bright, #4ade80)" },
  active: { body: "var(--vi-idle, #22c55e)", bright: "var(--vi-idle-bright, #4ade80)" },
  in_progress: { body: "var(--vi-work, #3b82f6)", bright: "var(--vi-work-bright, #60a5fa)" },
  waiting_for_approval: { body: "var(--vi-alert, #f97316)", bright: "var(--vi-alert-bright, #fb923c)" },
  waiting_for_answer: { body: "var(--vi-question, #c084fc)", bright: "var(--vi-question-bright, #d8b4fe)" },
  pending: { body: "var(--vi-idle, #22c55e)", bright: "var(--vi-idle-bright, #4ade80)" },
  completed: { body: "#44444a", bright: "#666" },
};

export function PixelPet({ status, size = 16 }: Props) {
  const colors = STATE_COLORS[status] || STATE_COLORS.idle;
  const scale = size / 8;

  return (
    <svg
      className="pixel-pet"
      width={size}
      height={size}
      viewBox="0 0 8 8"
      shapeRendering="crispEdges"
      style={{ imageRendering: "pixelated" }}
    >
      {/* Eyes */}
      <rect x="1" y="2" width="1" height="1" fill={colors.bright} />
      <rect x="4" y="2" width="1" height="1" fill={colors.bright} />

      {/* Body row 1 */}
      <rect x="0" y="3" width="6" height="1" fill={colors.body} />
      {/* Pupils */}
      <rect x="1" y="3" width="1" height="1" fill="#000" />
      <rect x="4" y="3" width="1" height="1" fill="#000" />

      {/* Body row 2 */}
      <rect x="0" y="4" width="6" height="1" fill={colors.body} />

      {/* Feet */}
      <rect x="1" y="5" width="2" height="1" fill={colors.body} />
      <rect x="4" y="5" width="2" height="1" fill={colors.body} />

      {/* Alert state: exclamation mark */}
      {(status === "waiting_for_approval" || status === "waiting_for_answer") && (
        <>
          <rect x="7" y="1" width="1" height="3" fill={colors.bright} />
          <rect x="7" y="5" width="1" height="1" fill={colors.bright} />
        </>
      )}
    </svg>
  );
}

/** Larger pet for the compact pill (26x16 original size) */
export function PixelPetLarge({ status }: { status: SessionStatus }) {
  const colors = STATE_COLORS[status] || STATE_COLORS.idle;

  return (
    <svg
      className="pixel-pet"
      width="26"
      height="16"
      viewBox="0 0 13 8"
      shapeRendering="crispEdges"
      style={{ imageRendering: "pixelated" }}
    >
      <rect x="2" y="2" width="1" height="1" fill={colors.bright} />
      <rect x="5" y="2" width="1" height="1" fill={colors.bright} />
      <rect x="1" y="3" width="6" height="1" fill={colors.body} />
      <rect x="2" y="3" width="1" height="1" fill="#000" />
      <rect x="5" y="3" width="1" height="1" fill="#000" />
      <rect x="1" y="4" width="6" height="1" fill={colors.body} />
      <rect x="2" y="5" width="2" height="1" fill={colors.body} />
      <rect x="5" y="5" width="2" height="1" fill={colors.body} />
    </svg>
  );
}
