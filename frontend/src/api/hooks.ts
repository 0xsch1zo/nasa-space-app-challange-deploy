import { useQuery } from '@tanstack/react-query';
import { api } from './client';
import type {
  Alert,
  CrewMember,
  MetricDefinition,
  MetricGroup,
  Overview,
  Series,
  WellbeingNote,
} from './types';

export function useCatalog() {
  return useQuery({
    queryKey: ['catalog'],
    queryFn: () => api<MetricDefinition[]>('/metrics'),
    staleTime: Infinity,
  });
}

export function useGroups() {
  return useQuery({
    queryKey: ['groups'],
    queryFn: () => api<MetricGroup[]>('/metric-groups'),
    staleTime: Infinity,
  });
}

export function useAlerts(id: string) {
  return useQuery({
    queryKey: ['alerts', id],
    queryFn: () => api<Alert[]>(`/astronauts/${id}/alerts`),
  });
}

export function useWellbeingNote(id: string) {
  return useQuery({
    queryKey: ['wellbeing-note', id],
    queryFn: () => api<WellbeingNote | null>(`/astronauts/${id}/wellbeing-note`),
  });
}

export function useCrew() {
  return useQuery({ queryKey: ['crew'], queryFn: () => api<CrewMember[]>('/astronauts') });
}

export function useAstronaut(id: string) {
  return useQuery({
    queryKey: ['astronaut', id],
    queryFn: () => api<CrewMember>(`/astronauts/${id}`),
  });
}

export function useOverview(id: string) {
  return useQuery({
    queryKey: ['overview', id],
    queryFn: () => api<Overview>(`/astronauts/${id}/overview`),
  });
}

export interface SeriesParams {
  from?: string;
  bucket?: 'hour' | 'day' | 'week';
}

export function useSeries(astronautId: string, metric: string, params: SeriesParams) {
  const qs = new URLSearchParams();
  if (params.from) qs.set('from', params.from);
  if (params.bucket) qs.set('bucket', params.bucket);
  return useQuery({
    queryKey: ['series', astronautId, metric, params],
    queryFn: () => api<Series>(`/astronauts/${astronautId}/metrics/${metric}/series?${qs}`),
  });
}
