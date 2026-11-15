import { Group, Paper, Text, ThemeIcon } from '@mantine/core';
import { IconLeaf } from '@tabler/icons-react';
import { useWellbeingNote } from '../../api/hooks';
import { QueryState } from '../../components/QueryState';
import { fmtDate } from '../../lib/dates';
import type { DashboardContext, WidgetModule } from '../types';

/**
 * Gently worded description of the astronaut's mental state. The note is
 * produced elsewhere and only displayed: deliberately no scores, statuses or
 * warning colours, so it never adds stress.
 */
function MentalNoteWidget({ ctx }: { ctx: DashboardContext }) {
  const note = useWellbeingNote(ctx.astronautId);
  return (
    <Paper withBorder p="lg" h="100%" bg="var(--mantine-color-grape-light)">
      <Group gap="sm" mb="sm" wrap="nowrap">
        <ThemeIcon color="grape" variant="light" radius="xl" size={36}>
          <IconLeaf size={20} />
        </ThemeIcon>
        <div>
          <Text fw={700}>Stan psychiczny</Text>
          <Text size="xs" c="dimmed">
            Krótka notka przygotowana na podstawie zebranych danych
          </Text>
        </div>
      </Group>
      <QueryState isLoading={note.isLoading} error={note.error} minHeight={60}>
        {note.data ? (
          <>
            <Text size="md" lh={1.6}>
              {note.data.body}
            </Text>
            <Text size="xs" c="dimmed" mt="sm">
              {fmtDate(note.data.writtenAt)}
            </Text>
          </>
        ) : (
          <Text c="dimmed">Notka pojawi się wkrótce.</Text>
        )}
      </QueryState>
    </Paper>
  );
}

export const mentalNoteWidget: WidgetModule = {
  id: 'mental_note',
  span: { base: 12 },
  Component: MentalNoteWidget,
};
