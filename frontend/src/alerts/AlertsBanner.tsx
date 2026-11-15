import { Group, Paper, Stack, Text, ThemeIcon, UnstyledButton } from '@mantine/core';
import { IconChevronRight, IconCircleCheck } from '@tabler/icons-react';
import type { Alert, MetricGroup } from '../api/types';
import { AlertItem } from './AlertItem';

/** How many of the most important alerts the banner shows. */
export const BANNER_LIMIT = 2;

/**
 * Top of the astronaut panel, above the content of every tab except
 * "Alerty": compact full-width banners with the `BANNER_LIMIT` most
 * important alerts (already sorted by severity on the backend), a green
 * "all good" bar when nothing is critical or a warning, and a link to the
 * full list when there are more. Kept compact (well under a third of the
 * screen height) so the tab content stays visible.
 */
export function AlertsBanner({
  alerts,
  groups,
  onOpenAlert,
  onShowAll,
}: {
  alerts: Alert[];
  groups: MetricGroup[];
  onOpenAlert: (alert: Alert) => void;
  onShowAll: () => void;
}) {
  const urgent = alerts.some((a) => a.severity !== 'notice');
  const shown = alerts.slice(0, BANNER_LIMIT);
  const hidden = alerts.length - shown.length;
  const groupName = (code: string) => groups.find((g) => g.code === code)?.name ?? code;

  return (
    <Stack gap={6} aria-label="Alerty zdrowotne" component="section">
      {!urgent && (
        <Paper
          py={6}
          px="sm"
          radius="md"
          bg="var(--mantine-color-teal-light)"
          style={{ border: '1px solid var(--mantine-color-teal-outline)' }}
        >
          <Group gap="sm" wrap="nowrap">
            <ThemeIcon color="teal" radius="xl" size={26}>
              <IconCircleCheck size={18} />
            </ThemeIcon>
            <div>
              <Text fw={700} size="sm" c="teal">
                Wszystkie parametry w normie
              </Text>
              <Text size="xs" c="dimmed">
                Brak alertów krytycznych i ostrzeżeń.
                {alerts.length > 0 &&
                  ` ${alerts.length} ${alerts.length === 1 ? 'uwaga organizacyjna' : 'uwagi organizacyjne'} poniżej.`}
              </Text>
            </div>
          </Group>
        </Paper>
      )}
      {shown.map((a) => (
        <AlertItem key={a.id} alert={a} groupName={groupName(a.group)} onOpen={onOpenAlert} variant="banner" />
      ))}
      {hidden > 0 && (
        <UnstyledButton onClick={onShowAll} aria-label="Pokaż wszystkie alerty">
          <Paper
            py={4}
            px="sm"
            radius="md"
            style={{ border: '1px dashed var(--mantine-color-default-border)' }}
          >
            <Group justify="center" gap={6}>
              <Text size="sm" fw={700}>
                + {hidden} {hidden === 1 ? 'kolejny alert' : hidden < 5 ? 'kolejne alerty' : 'kolejnych alertów'}
              </Text>
              <Text size="xs" c="dimmed">
                · zobacz wszystkie
              </Text>
              <IconChevronRight size={16} />
            </Group>
          </Paper>
        </UnstyledButton>
      )}
    </Stack>
  );
}
