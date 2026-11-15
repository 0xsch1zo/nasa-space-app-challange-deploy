import { fireEvent, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { MetricGroup } from '../api/types';
import { alert, anna, renderWithProviders } from '../test/utils';
import { HealthProfile } from './HealthProfile';
import { SidebarProvider, useSidebar } from './SidebarContext';

const groups: MetricGroup[] = [
  { code: 'environment', name: 'Środowisko zewnętrzne', description: '', default: false },
  { code: 'physiology', name: 'Parametry fizjologiczne', description: '', default: true },
  { code: 'wellbeing', name: 'Samopoczucie', description: '', default: false },
];
const alerts = [
  alert('muscle', 'critical', 'Zanik mięśni'),
  alert('radiation', 'warning', 'Promieniowanie na stacji'),
  alert('bone', 'warning', 'Zanik kości'),
];

const ok = <T,>(data: T) => ({ data, isLoading: false, error: null });

// Data hooks return fixed data; tab panels are stubbed (widgets are not under test).
vi.mock('../api/hooks', () => ({
  useAstronaut: () => ok(anna),
  useOverview: () => ok({ astronaut: anna, overallStatus: 'critical', metrics: [], body: [] }),
  useCatalog: () => ok([]),
  useGroups: () => ok(groups),
  useAlerts: () => ok(alerts),
}));
vi.mock('../dashboard/ProfileTabs', () => ({
  ALERTS_TAB: 'alerts',
  ProfileTabs: ({ tab }: { tab: string }) => <div data-testid="tab-panel">panel:{tab}</div>,
}));

/** Mirrors AppLayout: the side bar slot is rendered apart from the page. */
function Harness() {
  const { setSlot } = useSidebar();
  return (
    <>
      <aside data-testid="sidebar" ref={setSlot} />
      <main data-testid="main">
        <HealthProfile astronautId="a1" />
      </main>
    </>
  );
}

const renderProfile = (tab?: string) =>
  renderWithProviders(
    <SidebarProvider>
      <Harness />
    </SidebarProvider>,
    { route: tab ? `/me?tab=${tab}` : '/me' },
  );

describe('HealthProfile', () => {
  beforeEach(() => vi.clearAllMocks());

  it.each(['environment', 'physiology', 'wellbeing', 'alerts'])('has no profile card on the "%s" tab', (tab) => {
    renderProfile(tab);
    const main = screen.getByTestId('main');
    expect(within(main).getByTestId('tab-panel')).toHaveTextContent(`panel:${tab}`);
    expect(within(main).queryByRole('heading', { name: 'Anna Kowalska' })).not.toBeInTheDocument();
    expect(within(main).queryByText(/Dzień misji/)).not.toBeInTheDocument();
    expect(within(main).queryByText(/Start misji/)).not.toBeInTheDocument();
  });

  it('shows the alert banners (max 2) on parameter tabs', () => {
    renderProfile('physiology');
    const section = screen.getByRole('region', { name: 'Alerty zdrowotne' });
    expect(within(section).getAllByTestId('alert-banner')).toHaveLength(2);
  });

  it('hides the alert banners on the "Alerty" tab', () => {
    renderProfile('alerts');
    expect(screen.queryByRole('region', { name: 'Alerty zdrowotne' })).not.toBeInTheDocument();
    expect(screen.queryAllByTestId('alert-banner')).toHaveLength(0);
  });

  it('hides the banners after switching to "Alerty" via the "more" link and shows them again elsewhere', async () => {
    renderProfile('physiology');
    await userEvent.click(screen.getByRole('button', { name: 'Pokaż wszystkie alerty' }));
    expect(screen.getByTestId('tab-panel')).toHaveTextContent('panel:alerts');
    expect(screen.queryByRole('region', { name: 'Alerty zdrowotne' })).not.toBeInTheDocument();

    await userEvent.click(within(screen.getByTestId('sidebar')).getByRole('button', { name: /Samopoczucie/ }));
    expect(screen.getByRole('region', { name: 'Alerty zdrowotne' })).toBeInTheDocument();
  });

  it('puts the profile details in a tooltip on the name in the side bar', async () => {
    renderProfile('physiology');
    const sidebar = screen.getByTestId('sidebar');
    const trigger = within(sidebar).getByTestId('astronaut-profile-trigger');
    expect(trigger.tagName).toBe('BUTTON');
    expect(trigger).toHaveTextContent('Anna Kowalska');
    expect(document.getElementById(trigger.getAttribute('aria-describedby')!)).toHaveTextContent(
      'Dzień misji: 168',
    );

    // Keyboard focus opens it.
    await userEvent.tab();
    expect(trigger).toHaveFocus();
    const tooltip = await screen.findByRole('tooltip');
    expect(tooltip).toHaveTextContent('Inżynier pokładowy');
    expect(tooltip).toHaveTextContent('Start misji');
  });

  it('opens the side bar profile tooltip with a tap on touch devices', async () => {
    renderProfile('physiology');
    const trigger = within(screen.getByTestId('sidebar')).getByTestId('astronaut-profile-trigger');
    fireEvent.pointerUp(trigger, { pointerType: 'touch' });
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Dzień misji');
    // A tap elsewhere closes it.
    fireEvent.mouseDown(document.body);
    fireEvent.touchStart(document.body);
    await vi.waitFor(() => expect(screen.queryByRole('tooltip')).not.toBeInTheDocument());
  });
});
