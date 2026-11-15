import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { MetricDefinition, MetricGroup } from '../api/types';
import { anna, renderWithProviders } from '../test/utils';
import { CrewTable } from './CrewTable';

const groups: MetricGroup[] = [{ code: 'physiology', name: 'Parametry fizjologiczne', description: '', default: true }];
const catalog: MetricDefinition[] = [];

describe('profile tooltip in the supervisor view (crew table)', () => {
  it('describes the astronaut for assistive technology', () => {
    renderWithProviders(<CrewTable crew={[anna]} catalog={catalog} groups={groups} onOpen={() => {}} />);
    const trigger = screen.getByTestId('astronaut-profile-trigger');
    expect(trigger.tagName).toBe('BUTTON');
    const id = trigger.getAttribute('aria-describedby');
    expect(id).toBeTruthy();
    const description = document.getElementById(id!);
    expect(description).toHaveTextContent('Dzień misji: 168');
    expect(description).toHaveTextContent('Rola: Inżynier pokładowy');
    expect(description).toHaveTextContent('Stacja: Stacja Orbitalna Aurora');
  });

  it('opens on hover and keeps opening the profile on click', async () => {
    const onOpen = vi.fn();
    renderWithProviders(<CrewTable crew={[anna]} catalog={catalog} groups={groups} onOpen={onOpen} />);
    const trigger = screen.getByTestId('astronaut-profile-trigger');
    await userEvent.hover(trigger);
    const tooltip = await screen.findByRole('tooltip');
    expect(tooltip).toHaveTextContent('Start misji');
    expect(tooltip).toHaveTextContent('168');
    await userEvent.click(trigger);
    expect(onOpen).toHaveBeenCalled();
  });
});
