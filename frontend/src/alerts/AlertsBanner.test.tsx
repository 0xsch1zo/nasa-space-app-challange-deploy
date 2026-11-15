import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { MetricGroup } from '../api/types';
import { alert, renderWithProviders } from '../test/utils';
import { AlertsBanner, BANNER_LIMIT } from './AlertsBanner';

const groups: MetricGroup[] = [{ code: 'physiology', name: 'Parametry fizjologiczne', description: '', default: true }];

// Already sorted by severity, as returned by the backend.
const sorted = [
  alert('muscle', 'critical', 'Zanik mięśni'),
  alert('radiation', 'warning', 'Promieniowanie na stacji'),
  alert('bone', 'warning', 'Zanik kości'),
  alert('samples', 'notice', 'Próbki z powierzchni stacji'),
];

describe('AlertsBanner', () => {
  it('shows only the two most important alerts', () => {
    expect(BANNER_LIMIT).toBe(2);
    renderWithProviders(<AlertsBanner alerts={sorted} groups={groups} onOpenAlert={() => {}} onShowAll={() => {}} />);
    const banners = screen.getAllByTestId('alert-banner');
    expect(banners).toHaveLength(2);
    expect(banners[0]).toHaveTextContent('Zanik mięśni');
    expect(banners[0]).toHaveTextContent('Krytyczne');
    expect(banners[1]).toHaveTextContent('Promieniowanie na stacji');
    expect(screen.queryByText('Zanik kości')).not.toBeInTheDocument();
  });

  it('keeps severity, content and time in the compact layout', () => {
    renderWithProviders(<AlertsBanner alerts={sorted} groups={groups} onOpenAlert={() => {}} onShowAll={() => {}} />);
    const first = screen.getAllByTestId('alert-banner')[0];
    expect(within(first).getByText('Krytyczne')).toBeInTheDocument();
    expect(within(first).getByText('Zanik mięśni: wartość poza normą')).toBeInTheDocument();
    expect(within(first).getAllByText(/^pomiar /).length).toBeGreaterThan(0);
  });

  it('links the remaining alerts to the full list', async () => {
    const onShowAll = vi.fn();
    renderWithProviders(<AlertsBanner alerts={sorted} groups={groups} onOpenAlert={() => {}} onShowAll={onShowAll} />);
    const more = screen.getByRole('button', { name: 'Pokaż wszystkie alerty' });
    expect(more).toHaveTextContent('+ 2 kolejne alerty');
    await userEvent.click(more);
    expect(onShowAll).toHaveBeenCalledOnce();
  });

  it('has no "more" link when everything fits', () => {
    renderWithProviders(
      <AlertsBanner alerts={sorted.slice(0, 2)} groups={groups} onOpenAlert={() => {}} onShowAll={() => {}} />,
    );
    expect(screen.getAllByTestId('alert-banner')).toHaveLength(2);
    expect(screen.queryByRole('button', { name: 'Pokaż wszystkie alerty' })).not.toBeInTheDocument();
  });

  it('shows the green bar when there is nothing critical or a warning', () => {
    renderWithProviders(
      <AlertsBanner alerts={[sorted[3]]} groups={groups} onOpenAlert={() => {}} onShowAll={() => {}} />,
    );
    expect(screen.getByText('Wszystkie parametry w normie')).toBeInTheDocument();
    expect(screen.getAllByTestId('alert-banner')).toHaveLength(1);
  });
});
