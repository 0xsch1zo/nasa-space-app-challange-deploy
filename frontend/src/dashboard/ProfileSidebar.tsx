import { Badge, Divider, NavLink, Stack, Text } from '@mantine/core';
import {
  IconBell,
  IconChartLine,
  IconHeartbeat,
  IconMoodSmile,
  IconPlanet,
  type IconProps,
} from '@tabler/icons-react';
import type { ComponentType } from 'react';
import type { Alert, CrewMember, HealthStatus, MetricGroup } from '../api/types';
import { AstronautProfileTooltip } from '../components/AstronautProfileTooltip';
import { StatusBadge } from '../components/StatusBadge';
import { ALERTS_TAB } from './ProfileTabs';

/** Icon per group code; unknown (future) groups get a generic chart icon. */
const GROUP_ICONS: Record<string, ComponentType<IconProps>> = {
  environment: IconPlanet,
  physiology: IconHeartbeat,
  wellbeing: IconMoodSmile,
};

/**
 * Side navigation of a health profile: one entry per parameter group
 * (tab) with the group's worst status, plus the alerts list. The
 * astronaut's name at the top reveals the profile details in a tooltip.
 */
export function ProfileSidebar({
  astronaut,
  groups,
  groupStatuses,
  alerts,
  active,
  onSelect,
}: {
  astronaut: CrewMember | undefined;
  groups: MetricGroup[];
  groupStatuses: Record<string, HealthStatus>;
  alerts: Alert[];
  active: string;
  onSelect: (tab: string) => void;
}) {
  const urgent = alerts.filter((a) => a.severity !== 'notice').length;
  return (
    <Stack gap={4} component="nav" aria-label="Zakładki profilu">
      <Text size="xs" c="dimmed" tt="uppercase" fw={600} px="sm" pt={4}>
        Profil zdrowotny
      </Text>
      <div style={{ paddingInline: 'var(--mantine-spacing-sm)', paddingBottom: 'var(--mantine-spacing-xs)' }}>
        {astronaut ? (
          <AstronautProfileTooltip member={astronaut}>
            <Text fw={700} lineClamp={1} td="underline" style={{ textDecorationStyle: 'dotted', textUnderlineOffset: 3 }}>
              {astronaut.fullName}
            </Text>
          </AstronautProfileTooltip>
        ) : (
          <Text fw={700}>…</Text>
        )}
      </div>
      {groups.map((g) => {
        const Icon = GROUP_ICONS[g.code] ?? IconChartLine;
        const status = groupStatuses[g.code] ?? 'unknown';
        return (
          <NavLink
            key={g.code}
            label={g.name}
            leftSection={<Icon size={20} />}
            rightSection={status !== 'unknown' ? <StatusBadge status={status} size="xs" /> : null}
            active={active === g.code}
            onClick={() => onSelect(g.code)}
            variant="light"
            component="button"
            style={{ borderRadius: 'var(--mantine-radius-md)', width: '100%' }}
          />
        );
      })}
      <Divider my="xs" />
      <NavLink
        label="Alerty"
        description={alerts.length ? 'Pełna lista ze szczegółami' : 'Brak alertów'}
        leftSection={<IconBell size={20} />}
        rightSection={
          alerts.length > 0 ? (
            <Badge size="sm" circle={alerts.length < 10} color={urgent > 0 ? 'red' : 'blue'}>
              {alerts.length}
            </Badge>
          ) : null
        }
        active={active === ALERTS_TAB}
        onClick={() => onSelect(ALERTS_TAB)}
        variant="light"
        component="button"
        style={{ borderRadius: 'var(--mantine-radius-md)', width: '100%' }}
      />
    </Stack>
  );
}
