import { ThemeIcon } from '@mantine/core';
import { IconAlertOctagon, IconAlertTriangle, IconInfoCircle } from '@tabler/icons-react';
import type { AlertSeverity } from '../api/types';
import { severityColor, severityLabel } from '../lib/status';

const ICONS = {
  critical: IconAlertOctagon,
  warning: IconAlertTriangle,
  notice: IconInfoCircle,
} as const;

/** Icon signalling alert importance (shape + colour, so it is not colour-only). */
export function SeverityIcon({ severity, size = 34 }: { severity: AlertSeverity; size?: number }) {
  const Icon = ICONS[severity];
  return (
    <ThemeIcon
      color={severityColor[severity]}
      variant={severity === 'notice' ? 'light' : 'filled'}
      size={size}
      radius="xl"
      aria-label={severityLabel[severity]}
    >
      <Icon size={size * 0.6} />
    </ThemeIcon>
  );
}
