import type { ReactElement, ReactNode } from 'react';
import { MantineProvider } from '@mantine/core';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { render } from '@testing-library/react';
import type { Alert, CrewMember } from '../api/types';

/** Renders with the same providers as `main.tsx` (router optional entry). */
export function renderWithProviders(ui: ReactElement, { route = '/' }: { route?: string } = {}) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: ReactNode }) => (
    <MantineProvider>
      <QueryClientProvider client={client}>
        <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>
      </QueryClientProvider>
    </MantineProvider>
  );
  return render(ui, { wrapper: Wrapper });
}

export const anna: CrewMember = {
  id: 'a1',
  fullName: 'Anna Kowalska',
  agency: 'POLSA',
  specialty: 'Inżynier pokładowy',
  missionStart: '2026-06-01',
  stationId: 's1',
  stationName: 'Stacja Orbitalna Aurora',
  missionDay: 168,
  overallStatus: 'critical',
  lastMeasurementAt: new Date().toISOString(),
  metricStatuses: {},
  alerts: { critical: 1, warning: 2, notice: 1 },
};

export function alert(id: string, severity: Alert['severity'], title: string): Alert {
  return {
    id,
    severity,
    metric: id,
    group: 'physiology',
    title,
    message: `${title}: wartość poza normą`,
    recordedAt: new Date().toISOString(),
    findings: [],
  };
}
