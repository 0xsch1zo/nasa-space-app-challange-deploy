import { Badge, type BadgeProps } from '@mantine/core';
import type { HealthStatus } from '../api/types';
import { statusColor, statusLabel } from '../lib/status';

export function StatusBadge({ status, ...props }: { status: HealthStatus } & BadgeProps) {
  return (
    <Badge color={statusColor[status]} variant="light" style={{ flexShrink: 0, overflow: 'visible' }} {...props}>
      {statusLabel[status]}
    </Badge>
  );
}
