// The admin's request list: its status filter and the badge of a
// resolved request.
export const STATUS_FILTERS = ['pending', 'done', 'rejected'] as const;

export type StatusFilter = (typeof STATUS_FILTERS)[number];

export function statusFilterOf(status: string): StatusFilter {
  return STATUS_FILTERS.includes(status as StatusFilter) ? (status as StatusFilter) : 'pending';
}

export function statusVariant(status: string): 'secondary' | 'outline' | 'destructive' {
  if (status === 'done') {
    return 'secondary';
  }
  if (status === 'rejected') {
    return 'destructive';
  }
  return 'outline';
}
