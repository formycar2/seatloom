import React from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { useDataStore } from '../stores/useDataStore';
import DispatchInboxView from './DispatchInboxView';
import ArtifactsView from './ArtifactsView';
afterEach(cleanup);
describe('Mira no-project guards combined with scoped backend', () => {
  it('does not display a default inbox when no project is selected', () => {
    useDataStore.setState({ activeProjectId: null, backendHydrated: true });
    render(<DispatchInboxView />);
    expect(screen.getByRole('status').textContent).toContain('请选择');
  });
  it('does not present unread project data as an inbox', () => {
    useDataStore.setState({ activeProjectId: 'seatloom', backendHydrated: false });
    render(<DispatchInboxView />);
    expect(screen.getByRole('status').textContent).toContain('未读就绪');
  });
  it('discloses the unscoped runtime-artifact half to the operator, not only in comments', () => {
    useDataStore.setState({ activeProjectId: 'seatloom', projectData: { seatloom: {
      seats: [], workItems: [], handoffs: [], sessions: [], events: [], inboxItems: [], artifacts: [],
    } } });
    render(<ArtifactsView onSelectArtifact={() => {}} />);
    expect(screen.getByRole('status').textContent).toContain('全局数据');
    expect(screen.getByRole('status').textContent).toContain('不代表属于当前项目');
  });
});
