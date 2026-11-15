import { useEffect, useState } from 'react';
import { CloseButton, Group, Tabs, Text } from '@mantine/core';
import { IconUser, IconUsersGroup } from '@tabler/icons-react';
import { useNavigate, useParams } from 'react-router-dom';
import { useCatalog, useCrew, useGroups } from '../api/hooks';
import { CrewTable } from '../components/CrewTable';
import { HealthProfile } from '../components/HealthProfile';
import { QueryState } from '../components/QueryState';

const CREW_TAB = 'crew';

/**
 * Supervisor workspace: a "Crew" tab with everyone on the station plus one
 * closable tab per opened astronaut profile. The active tab is the URL.
 */
export function SupervisorPage() {
  const { astronautId } = useParams();
  const navigate = useNavigate();
  const crew = useCrew();
  const catalog = useCatalog();
  const groups = useGroups();
  const [openIds, setOpenIds] = useState<string[]>([]);

  useEffect(() => {
    if (astronautId && !openIds.includes(astronautId)) setOpenIds((ids) => [...ids, astronautId]);
  }, [astronautId, openIds]);

  const active = astronautId ?? CREW_TAB;
  const nameOf = (id: string) => crew.data?.find((c) => c.id === id)?.fullName ?? 'Profil';

  function close(id: string) {
    setOpenIds((ids) => ids.filter((x) => x !== id));
    if (active === id) navigate('/crew');
  }

  return (
    <Tabs
      value={active}
      onChange={(v) => navigate(v && v !== CREW_TAB ? `/crew/${v}` : '/crew')}
      keepMounted={false}
    >
      <Tabs.List mb="md">
        <Tabs.Tab value={CREW_TAB} leftSection={<IconUsersGroup size={16} />}>
          Załoga
        </Tabs.Tab>
        {openIds.map((id) => (
          <Tabs.Tab key={id} value={id} leftSection={<IconUser size={16} />}>
            <Group gap={6} wrap="nowrap">
              <Text size="sm">{nameOf(id)}</Text>
              <CloseButton
                size="xs"
                component="span"
                aria-label="Zamknij kartę"
                onClick={(e) => {
                  e.stopPropagation();
                  close(id);
                }}
              />
            </Group>
          </Tabs.Tab>
        ))}
      </Tabs.List>

      <Tabs.Panel value={CREW_TAB}>
        <QueryState
          isLoading={crew.isLoading || catalog.isLoading || groups.isLoading}
          error={crew.error ?? catalog.error ?? groups.error}
        >
          {crew.data && catalog.data && groups.data && (
            <CrewTable
              crew={crew.data}
              catalog={catalog.data}
              groups={groups.data}
              onOpen={(c) => navigate(`/crew/${c.id}`)}
            />
          )}
        </QueryState>
      </Tabs.Panel>
      {openIds.map((id) => (
        <Tabs.Panel key={id} value={id}>
          <HealthProfile astronautId={id} />
        </Tabs.Panel>
      ))}
    </Tabs>
  );
}
