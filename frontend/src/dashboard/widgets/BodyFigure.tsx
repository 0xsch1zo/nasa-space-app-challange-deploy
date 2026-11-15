import type { MouseEvent } from 'react';

/**
 * Schematic human figure (front + back) with individually addressable
 * anatomical parts. Part keys are the backend `BodyRegion` names, so a metric
 * component with `bodyRegion: "quadriceps"` lights up the quadriceps.
 * Adding a part = one entry in `PARTS`.
 */

type Shape =
  | { kind: 'rect'; x: number; y: number; w: number; h: number; r: number }
  | { kind: 'ellipse'; cx: number; cy: number; rx: number; ry: number };

const rect = (x: number, y: number, w: number, h: number, r: number): Shape => ({ kind: 'rect', x, y, w, h, r });
const ellipse = (cx: number, cy: number, rx: number, ry: number): Shape => ({ kind: 'ellipse', cx, cy, rx, ry });
const pair = (left: Shape, dx: number): Shape[] =>
  left.kind === 'rect' ? [left, { ...left, x: left.x + dx }] : [left, { ...left, cx: left.cx + dx }];

type View = 'front' | 'back';

// Silhouette (160 × 410), identical for both views.
const SILHOUETTE: Shape[] = [
  ellipse(80, 36, 21, 26),
  rect(71, 60, 18, 14, 4),
  ...pair(rect(24, 78, 18, 128, 9), 94),
  rect(46, 72, 68, 124, 16),
  rect(47, 198, 66, 34, 14),
  ...pair(rect(50, 234, 27, 90, 12), 33),
  ...pair(rect(52, 328, 23, 78, 10), 33),
];

export const PARTS: Record<string, { view: View; shapes: Shape[] }> = {
  // general
  head: { view: 'front', shapes: [ellipse(80, 36, 21, 26)] },
  heart: { view: 'front', shapes: [ellipse(93, 102, 10, 11)] },
  chest: { view: 'front', shapes: [rect(46, 72, 68, 72, 16)] },
  abdomen: { view: 'front', shapes: [rect(49, 146, 62, 50, 10)] },
  arms: { view: 'front', shapes: pair(rect(24, 78, 18, 128, 9), 94) },
  // muscles
  deltoids: { view: 'front', shapes: pair(ellipse(36, 86, 13, 14), 88) },
  quadriceps: { view: 'front', shapes: pair(rect(52, 240, 23, 74, 10), 33) },
  erector_spinae: { view: 'back', shapes: pair(rect(65, 88, 11, 106, 5), 19) },
  gluteus: { view: 'back', shapes: pair(ellipse(64, 213, 16, 15), 32) },
  hamstrings: { view: 'back', shapes: pair(rect(52, 240, 23, 76, 10), 33) },
  calves: { view: 'back', shapes: pair(rect(54, 334, 19, 56, 9), 33) },
  // bones
  lumbar_spine: { view: 'back', shapes: [rect(75, 150, 10, 46, 3)] },
  pelvis: { view: 'front', shapes: [rect(54, 200, 52, 26, 12)] },
  femoral_neck: { view: 'front', shapes: pair(ellipse(61, 236, 8, 6), 38) },
  distal_tibia: { view: 'front', shapes: pair(rect(57, 380, 13, 22, 4), 33) },
  distal_radius: { view: 'front', shapes: pair(rect(27, 176, 12, 26, 4), 94) },
};

export interface PartStyle {
  /** CSS colour */
  fill: string;
  label: string;
}

export function BodyFigure({
  parts,
  selected,
  onSelect,
  onHover,
}: {
  /** Parts to highlight, keyed by body region. */
  parts: Record<string, PartStyle>;
  selected: string | null;
  onSelect: (part: string) => void;
  onHover?: (part: string | null, event?: MouseEvent<SVGElement>) => void;
}) {
  const draw = (s: Shape, key: string, props: Record<string, unknown>) =>
    s.kind === 'rect' ? (
      <rect key={key} x={s.x} y={s.y} width={s.w} height={s.h} rx={s.r} {...props} />
    ) : (
      <ellipse key={key} cx={s.cx} cy={s.cy} rx={s.rx} ry={s.ry} {...props} />
    );

  const figure = (view: View) => (
    <>
      {SILHOUETTE.map((s, i) =>
        draw(s, `sil-${i}`, {
          fill: 'var(--mantine-color-gray-6)',
          fillOpacity: 0.16,
          stroke: 'var(--mantine-color-default-border)',
        }),
      )}
      {Object.entries(parts).flatMap(([part, style]) => {
        const def = PARTS[part];
        if (!def || def.view !== view) return [];
        const isSelected = part === selected;
        return def.shapes.map((s, i) =>
          draw(s, `${part}-${i}`, {
            fill: style.fill,
            fillOpacity: isSelected ? 1 : 0.8,
            stroke: isSelected ? 'var(--mantine-color-text)' : 'var(--mantine-color-body)',
            strokeWidth: isSelected ? 2.5 : 1,
            style: { cursor: 'pointer', transition: 'fill-opacity 150ms' },
            role: 'button',
            tabIndex: 0,
            'aria-label': style.label,
            onClick: () => onSelect(part),
            onKeyDown: (e: React.KeyboardEvent) => e.key === 'Enter' && onSelect(part),
            onMouseMove: (e: MouseEvent<SVGElement>) => onHover?.(part, e),
            onMouseLeave: () => onHover?.(null),
          }),
        );
      })}
    </>
  );

  return (
    <svg viewBox="0 0 360 440" width="100%" style={{ maxHeight: 400 }} role="img" aria-label="Sylwetka ciała">
      <g>{figure('front')}</g>
      <g transform="translate(190 0)">{figure('back')}</g>
      <text x={80} y={432} textAnchor="middle" fontSize={13} fill="var(--mantine-color-dimmed)">
        Przód
      </text>
      <text x={270} y={432} textAnchor="middle" fontSize={13} fill="var(--mantine-color-dimmed)">
        Tył
      </text>
    </svg>
  );
}
