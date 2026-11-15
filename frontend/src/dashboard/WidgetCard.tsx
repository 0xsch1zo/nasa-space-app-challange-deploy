import { Group, Paper, Text } from '@mantine/core';
import type { ReactNode } from 'react';

/** Common frame so independently developed widgets look consistent. */
export function WidgetCard({
  title,
  subtitle,
  right,
  children,
}: {
  title: string;
  subtitle?: string;
  right?: ReactNode;
  children: ReactNode;
}) {
  return (
    <Paper withBorder p="md" h="100%">
      <Group justify="space-between" align="flex-start" mb="sm" wrap="nowrap">
        <div style={{ minWidth: 0 }}>
          <Text fw={700}>{title}</Text>
          {subtitle && (
            <Text size="xs" c="dimmed">
              {subtitle}
            </Text>
          )}
        </div>
        {right && <div style={{ flexShrink: 0 }}>{right}</div>}
      </Group>
      {children}
    </Paper>
  );
}
