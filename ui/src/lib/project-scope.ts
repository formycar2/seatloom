// An async result is publishable only for the selection that issued it.
export function projectStillSelected(projectId: string, current: string | null): boolean {
  return current === projectId;
}

// Use the registered seat names, not every tmux session or a project-name heuristic.
export function boundMirrorNames<T extends { seatName: string }>(
  discovered: T[], boundSeats: { name: string }[],
): T[] {
  const names = new Set(boundSeats.map((s) => s.name.toLowerCase()));
  return discovered.filter((s) => names.has(s.seatName.toLowerCase()));
}
