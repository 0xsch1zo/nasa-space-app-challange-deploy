import { useId, useState, type ReactNode } from 'react';
import { Group, Stack, Text, Tooltip, UnstyledButton, VisuallyHidden } from '@mantine/core';
import { useClickOutside } from '@mantine/hooks';
import type { CrewMember } from '../api/types';
import { fmtDate, fromNow } from '../lib/dates';
import { statusLabel } from '../lib/status';
import { StatusBadge } from './StatusBadge';

/**
 * Profile facts of an astronaut (formerly the profile card shown on every
 * tab). Single source for the visual tooltip and its accessible description.
 */
export function astronautProfileFacts(member: CrewMember): { label: string; value: string }[] {
  return [
    { label: 'Rola', value: member.specialty },
    { label: 'Agencja', value: member.agency },
    { label: 'Stacja', value: member.stationName },
    { label: 'Dzień misji', value: String(member.missionDay) },
    { label: 'Start misji', value: fmtDate(member.missionStart) },
    {
      label: 'Ostatni pomiar',
      value: member.lastMeasurementAt ? fromNow(member.lastMeasurementAt) : '—',
    },
    { label: 'Stan ogólny', value: statusLabel[member.overallStatus] },
  ];
}

/** Content of the profile tooltip. */
export function AstronautProfileDetails({ member }: { member: CrewMember }) {
  return (
    <Stack gap={6}>
      <Group justify="space-between" wrap="nowrap" gap="sm">
        <Text fw={700} size="sm">
          {member.fullName}
        </Text>
        <StatusBadge status={member.overallStatus} size="xs" />
      </Group>
      <Stack gap={2}>
        {astronautProfileFacts(member)
          .filter((f) => f.label !== 'Stan ogólny')
          .map((f) => (
            <Group key={f.label} justify="space-between" wrap="nowrap" gap="md">
              <Text size="xs" c="dimmed">
                {f.label}
              </Text>
              <Text size="xs" fw={600} ta="right">
                {f.value}
              </Text>
            </Group>
          ))}
      </Stack>
    </Stack>
  );
}

/**
 * Interactive trigger (a button) showing the astronaut's profile in a
 * tooltip on mouse hover and keyboard focus. On touch devices a tap toggles
 * it and a tap elsewhere closes it — unless the trigger has its own action
 * (`onClick`, e.g. opening the profile), which a tap then performs. The same
 * facts are always available to assistive technology via `aria-describedby`.
 */
export function AstronautProfileTooltip({
  member,
  children,
  onClick,
  position = 'right-start',
}: {
  member: CrewMember;
  children: ReactNode;
  onClick?: () => void;
  position?: 'right-start' | 'bottom-start';
}) {
  const descriptionId = useId();
  // Touch has no hover: a tap opens the tooltip in controlled mode.
  const [tapped, setTapped] = useState(false);
  const outsideRef = useClickOutside<HTMLButtonElement>(() => setTapped(false));
  return (
    <>
      <Tooltip
        opened={tapped ? true : undefined}
        label={<AstronautProfileDetails member={member} />}
        events={{ hover: true, focus: true, touch: true }}
        position={position}
        withArrow
        multiline
        w={260}
        openDelay={150}
        color="dark"
      >
        <UnstyledButton
          ref={outsideRef}
          onClick={onClick}
          onPointerUp={(e) => {
            if (e.pointerType === 'touch' && !onClick) setTapped((v) => !v);
          }}
          onBlur={() => setTapped(false)}
          aria-describedby={descriptionId}
          data-testid="astronaut-profile-trigger"
          style={{ borderRadius: 'var(--mantine-radius-sm)', maxWidth: '100%' }}
        >
          {children}
        </UnstyledButton>
      </Tooltip>
      <VisuallyHidden id={descriptionId}>
        {astronautProfileFacts(member)
          .map((f) => `${f.label}: ${f.value}`)
          .join('. ')}
      </VisuallyHidden>
    </>
  );
}
