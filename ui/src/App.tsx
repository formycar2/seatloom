import React, { useEffect, useMemo, useRef, useState } from 'react';
import AppShell from './layouts/AppShell';
import MasterDetail from './layouts/MasterDetail';
import InboxView from './views/InboxView';
import TimelineView from './views/TimelineView';
import DispatchInboxView from './views/DispatchInboxView';
import WorkItemsView from './views/WorkItemsView';
import SeatsView from './views/SeatsView';
import ArtifactsView from './views/ArtifactsView';
import PlaybookView from './views/PlaybookView';
import AllProjectsView from './views/AllProjectsView';
import LiveSeatsView from './views/LiveSeatsView';
import SupervisionDashboard from './components/SupervisionDashboard';
import SeatDetail from './components/SeatDetail';
import SessionDetail from './components/SessionDetail';
import WorkItemDetail from './components/WorkItemDetail';
import HandoffDetail from './components/HandoffDetail';
import ArtifactDetail from './components/ArtifactDetail';
import ProjectOverview from './components/ProjectOverview';
import WorkItemForm from './components/WorkItemForm';
import HandoffForm from './components/HandoffForm';
import AddSeatDialog from './components/AddSeatDialog';
import InitDialog from './components/InitDialog';
import ShortcutHelpDialog from './components/ShortcutHelpDialog';
import SupervisorCommandBar from './components/SupervisorCommandBar';
import { SupervisorPanel } from './app-v2/panel/SupervisorPanel';
import { SessionsWorkspace } from './app-v2/panel/SessionsWorkspace';
import { IMErrorBoundary } from './supervisor/IMErrorBoundary';
import { api, isTauri, onSupervisorDetached, onSupervisorReembedded } from './lib/api';
import PipelineProgress from './components/PipelineProgress';
import ReconcileNotification from './components/ReconcileNotification';
import SwitchProtectionDialog from './components/SwitchProtectionDialog';
import { useDataStore } from './stores/useDataStore';
import { useAppStore } from './stores/useAppStore';
import { useLiveSessionsStore } from './stores/useLiveSessionsStore';
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts';
import { ProvenanceFrame } from './components/ProvenanceBanner';
import { provenanceForTab } from './lib/provenance';
import { ArtifactSubtype, ArtifactTemplate, SelectedObjectType } from './types';

type Tab =
  | 'live-seats'
  | 'dashboard'
  | 'inbox'
  | 'timeline'
  | 'dispatches'
  | 'workitems'
  | 'seats'
  | 'artifacts'
  | 'playbook'
  | 'sessions'
  | 'all-projects';
type FormView = 'none' | 'createWorkItem' | 'createHandoff' | 'pipelineProgress' | 'addSeat';
type SelectedObject = { type: SelectedObjectType; id: string } | null;

const App: React.FC = () => {
  const {
    projectData,
    activeProjectId,
    setActiveProject,
    addWorkItem,
    hydrateFromBackend,
    refreshEvents,
    backendHydrated,
  } = useDataStore();
  // Live seat discovery status (UM0 HOLD-3): the provenance banner must not
  // claim LIVE while tmux discovery is empty or has failed.
  const seatDiscovery = useLiveSessionsStore((s) => s.seatDiscovery);
  const { projectUIStates, updateProjectUIState } = useAppStore();
  const projectUIStatesRef = useRef(projectUIStates);
  useEffect(() => {
    projectUIStatesRef.current = projectUIStates;
  });

  // Hydrate from backend on first mount. Silently noops in browser-dev.
  useEffect(() => {
    hydrateFromBackend();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Live timeline: poll the active project's events every 8s so new captured
  // events (incl. MCP-emitted ones from other processes) appear without a reload.
  useEffect(() => {
    const timer = setInterval(() => {
      void refreshEvents();
    }, 8000);
    return () => clearInterval(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 张先生 2026-10-16 定下第一屏是什么:「全队在干什么、什么卡着,什么需要我决策」。
  // 派单页是产品里唯一一张按真实表读出来的界面(177 条派单、实时状态、真实依赖边,每 5 秒轮询),
  // 而且它已经直接回答那个问题:按席位分组、每组一条状态带、每个席位一个固定颜色。
  // 实时席位那一页保留在 NavRail 里,作为「钻进某个席位原始终端」的细节视图 —— 他说过镜像是
  // 过渡态,不是说要删掉。契约:LYRA-2026-10-16-first-screen-contract-v1 §6.2 slice 1。
  const [activeTab, setActiveTab] = useState<Tab>('dispatches');
  const [selectedObject, setSelectedObject] = useState<SelectedObject>(null);
  const [showHelp, setShowHelp] = useState(false);
  const [showCommandBar, setShowCommandBar] = useState(false);
  const [showSupervisorIM, setShowSupervisorIM] = useState(
    () => localStorage.getItem('seatloom.supervisor.open') === 'true',
  );
  const [supervisorDetached, setSupervisorDetached] = useState(false);

  useEffect(() => {
    localStorage.setItem('seatloom.supervisor.open', showSupervisorIM ? 'true' : 'false');
  }, [showSupervisorIM]);

  // Reflect the current detached/embedded state from the backend on mount so
  // reopening the main window after detaching the IM doesn't re-show the
  // embedded panel next to the detached window.
  useEffect(() => {
    if (!isTauri()) return;
    api
      .supervisorWindowStatus()
      .then((s) => setSupervisorDetached(s.detached))
      .catch(() => {});
    let offDetached: (() => void) | null = null;
    let offReembed: (() => void) | null = null;
    onSupervisorDetached(() => {
      setSupervisorDetached(true);
      setShowSupervisorIM(false);
    }).then((fn) => {
      offDetached = fn;
    });
    onSupervisorReembedded(() => {
      setSupervisorDetached(false);
    }).then((fn) => {
      offReembed = fn;
    });
    return () => {
      if (offDetached) offDetached();
      if (offReembed) offReembed();
    };
  }, []);

  // ⌘K toggles the Supervisor IM — the L1 surface. The V1 command bar is
  // still registered on ⌘⇧K for backwards compatibility with existing
  // global shortcuts. If the IM is detached, ⌘K focuses that window
  // instead of reopening the embedded panel.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'k') {
        e.preventDefault();
        if (supervisorDetached && isTauri()) {
          api.openSupervisorWindow().catch(() => {});
          return;
        }
        setShowSupervisorIM((v) => !v);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [supervisorDetached]);

  const [activeForm, setActiveForm] = useState<FormView>('none');
  const [isInitOpen, setIsInitOpen] = useState(false);
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [showReconcileSummary, setShowReconcileSummary] = useState(false);
  const currentProjectData = activeProjectId ? projectData[activeProjectId] : null;
  const reconcileIssueCount = useMemo(
    () => currentProjectData?.workItems.filter((w) => w.status === 'Blocked').length ?? 0,
    [currentProjectData?.workItems],
  );

  const [timelineFilters, setTimelineFilters] = useState<{
    seatId: string | null;
    workItemId: string | null;
    eventType: string | null;
    artifactTemplate: ArtifactTemplate | null;
    artifactSubtype: ArtifactSubtype | null;
    timeRange: string;
  }>({
    seatId: null,
    workItemId: null,
    eventType: null,
    artifactTemplate: null,
    artifactSubtype: null,
    timeRange: '24h',
  });

  const [pendingProjectId, setPendingProjectId] = useState<string | null>(null);
  const [protectionType, setProtectionType] = useState<'session' | 'form' | null>(null);

  useEffect(() => {
    if (!activeProjectId && activeTab !== 'all-projects' && activeTab !== 'live-seats') {
      setIsInitOpen(true);
    }
  }, [activeProjectId, activeTab]);

  useGlobalShortcuts(
    () => {
      setSelectedObject(null);
      setActiveForm('none');
      setShowCommandBar(false);
    },
    () => {},
    () => setIsTerminalOpen((prev) => !prev),
    () => setShowHelp(true),
    () => setShowCommandBar(true),
  );

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects') {
      const savedState = projectUIStatesRef.current[activeProjectId];
      if (savedState) {
        setActiveTab(savedState.activeTab as Tab);
        setSelectedObject(
          savedState.selectedObjectId && savedState.selectedObjectType
            ? { type: savedState.selectedObjectType, id: savedState.selectedObjectId }
            : null,
        );
      }
    }
  }, [activeProjectId]);

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects' && activeTab !== 'all-projects') {
      updateProjectUIState(activeProjectId, {
        activeTab,
        selectedObjectId: selectedObject?.id || null,
        selectedObjectType: selectedObject?.type || null,
      });
    }
  }, [activeTab, selectedObject, activeProjectId, updateProjectUIState]);

  // ─── Project switching ───
  const handleProjectSwitchRequest = (id: string) => {
    if (id === 'all-projects') {
      setActiveTab('all-projects');
      return;
    }
    const currentData = activeProjectId ? projectData[activeProjectId] : null;
    const hasRunningSessions = currentData?.sessions.some(
      (s) => s.status === 'Running' || s.status === 'InputRequired',
    );
    const hasUnsavedInput = activeForm !== 'none' && activeForm !== 'pipelineProgress';
    if (hasRunningSessions) {
      setPendingProjectId(id);
      setProtectionType('session');
      return;
    }
    if (hasUnsavedInput) {
      setPendingProjectId(id);
      setProtectionType('form');
      return;
    }
    performSwitch(id);
  };

  const performSwitch = (id: string) => {
    setActiveProject(id);
    setPendingProjectId(null);
    setProtectionType(null);
    if (activeTab === 'all-projects') setActiveTab('dashboard');
  };

  // ─── Object selection ───
  const handleSelectObject = (type: string, data: any) => {
    if (type === 'Action' && data === 'AddSeat') {
      setActiveForm('addSeat');
      return;
    }
    if (type === 'Action' && data === 'AddWorkItem') {
      setActiveForm('createWorkItem');
      return;
    }
    const id = typeof data === 'string' ? data : data.id;
    setSelectedObject({ type: type as any, id });
    setActiveForm('none');
    if (type === 'Session') {
      const session =
        typeof data === 'string' ? currentProjectData?.sessions.find((s) => s.id === data) : data;
      if (session && (session.status === 'Running' || session.status === 'InputRequired'))
        setIsTerminalOpen(true);
    }
  };

  const handleOpenArtifact = (artifactId: string) => {
    setSelectedObject({ type: 'Artifact', id: artifactId });
    setActiveForm('none');
  };

  const handleInboxSelect = (item: any) => {
    const current = activeProjectId ? projectData[activeProjectId] : null;
    if (!current) return;
    const ref = item.object_ref?.toLowerCase() || '';
    if (ref.startsWith('ho-')) {
      const h = current.handoffs.find((e: any) => e.id.toLowerCase() === ref);
      if (h) {
        setSelectedObject({ type: 'Handoff', id: h.id });
        setActiveForm('none');
        return;
      }
    }
    if (ref.startsWith('wi-')) {
      const w = current.workItems.find((e: any) => e.id.toLowerCase() === ref);
      if (w) {
        setSelectedObject({ type: 'WorkItem', id: w.id });
        setActiveForm('none');
        return;
      }
    }
    if (ref.startsWith('ses-')) {
      const s = current.sessions.find((e: any) => e.id.toLowerCase() === ref);
      if (s) {
        setSelectedObject({ type: 'Session', id: s.id });
        setActiveForm('none');
        if (s.status === 'Running' || s.status === 'InputRequired') setIsTerminalOpen(true);
        return;
      }
    }
    if (ref.startsWith('ar-')) {
      const a = current.artifacts.find((e: any) => e.id.toLowerCase() === ref);
      if (a) {
        setSelectedObject({ type: 'Artifact', id: a.id });
        setActiveForm('none');
        return;
      }
    }
    const linkedId = item.linked_artifact_ids?.find((id: string) =>
      current.artifacts.some((a: any) => a.id === id),
    );
    if (linkedId) {
      setSelectedObject({ type: 'Artifact', id: linkedId });
      setActiveForm('none');
      return;
    }
    setActiveForm('none');
  };

  const handleConfirmProposal = (workItem: any) => {
    addWorkItem(workItem);
    setSelectedObject({ type: 'WorkItem', id: workItem.id });
    setActiveTab('workitems');
    setActiveForm('none');
  };

  const clearSelection = () => {
    setSelectedObject(null);
    setActiveForm('none');
  };

  // ─── Data ───
  const resolvedData = useMemo(() => {
    if (!selectedObject || !currentProjectData) return null;
    switch (selectedObject.type) {
      case 'Seat':
        return currentProjectData.seats.find((s) => s.id === selectedObject.id) ?? null;
      case 'Session':
        return currentProjectData.sessions.find((s) => s.id === selectedObject.id) ?? null;
      case 'WorkItem':
        return currentProjectData.workItems.find((w) => w.id === selectedObject.id) ?? null;
      case 'Artifact':
        return currentProjectData.artifacts.find((a) => a.id === selectedObject.id) ?? null;
      case 'Handoff':
        return currentProjectData.handoffs.find((h) => h.id === selectedObject.id) ?? null;
      default:
        return null;
    }
  }, [selectedObject, currentProjectData]);

  // ─── Detail content renderer ───
  const renderDetailContent = () => {
    if (activeForm === 'pipelineProgress') return <PipelineProgress />;
    if (activeForm === 'createWorkItem')
      return <WorkItemForm onClose={() => setActiveForm('none')} />;
    if (activeForm === 'createHandoff')
      return (
        <HandoffForm
          onClose={() => setActiveForm('none')}
          workItemId={selectedObject?.type === 'WorkItem' ? selectedObject.id : undefined}
        />
      );
    if (!selectedObject || !resolvedData) return <ProjectOverview />;
    switch (selectedObject.type) {
      case 'Seat':
        return <SeatDetail seat={resolvedData as any} />;
      case 'Session':
        return <SessionDetail session={resolvedData as any} onOpenArtifact={handleOpenArtifact} />;
      case 'WorkItem':
        return (
          <WorkItemDetail workItem={resolvedData as any} onOpenArtifact={handleOpenArtifact} />
        );
      case 'Artifact':
        return <ArtifactDetail artifact={resolvedData as any} />;
      case 'Handoff':
        return <HandoffDetail handoff={resolvedData as any} onOpenArtifact={handleOpenArtifact} />;
      default:
        return null;
    }
  };

  // ─── Main view renderer ───
  const renderMainView = () => {
    if (activeTab === 'all-projects') {
      return <AllProjectsView onSelectProject={handleProjectSwitchRequest} />;
    }
    if (activeTab === 'live-seats') {
      return <LiveSeatsView />;
    }
    if (activeTab === 'sessions') {
      return <SessionsWorkspace />;
    }
    if (activeTab === 'dashboard') {
      return (
        <SupervisionDashboard
          onNavigateToInbox={() => {
            setActiveTab('inbox');
            setSelectedObject(null);
          }}
          onSelectInboxItem={handleInboxSelect}
        />
      );
    }

    // All other tabs use MasterDetail layout
    const hasSelection = !!(selectedObject && resolvedData);

    const listContent = (() => {
      switch (activeTab) {
        case 'inbox':
          return (
            <InboxView onSelectObject={handleInboxSelect} selectedObjectId={selectedObject?.id} />
          );
        case 'timeline':
          return (
            <TimelineView
              onSelectObject={handleSelectObject}
              onOpenArtifact={handleOpenArtifact}
              filters={timelineFilters}
              onFiltersChange={setTimelineFilters}
            />
          );
        case 'dispatches':
          return <DispatchInboxView />;
        case 'workitems':
          return (
            <WorkItemsView
              onSelectWI={(wi) =>
                wi === ('Action:AddWorkItem' as any)
                  ? handleSelectObject('Action', 'AddWorkItem')
                  : handleSelectObject('WorkItem', wi)
              }
            />
          );
        case 'seats':
          return (
            <SeatsView
              onSelectSeat={(s) => handleSelectObject('Seat', s)}
              onSelectSession={(s) => handleSelectObject('Session', s)}
              selectedId={selectedObject?.id}
            />
          );
        case 'artifacts':
          return (
            <ArtifactsView
              onSelectArtifact={(a) => handleSelectObject('Artifact', a)}
              selectedId={selectedObject?.id}
            />
          );
        case 'playbook':
          return (
            <PlaybookView
              onSelectArtifact={(a) => handleSelectObject('Artifact', a)}
              selectedId={selectedObject?.id}
            />
          );
        default:
          return null;
      }
    })();

    return (
      <MasterDetail
        list={listContent}
        detail={hasSelection ? renderDetailContent() : null}
        hasSelection={hasSelection}
        onBack={clearSelection}
      />
    );
  };

  return (
    <>
      <AppShell
        activeTab={activeTab}
        onTabChange={(tab) => {
          setActiveTab(tab as Tab);
          setSelectedObject(null);
          setActiveForm('none');
        }}
        onSelectProject={handleProjectSwitchRequest}
        onInitProject={() => setIsInitOpen(true)}
        onShowHelp={() => setShowHelp(true)}
        isTerminalOpen={isTerminalOpen}
        onTerminalToggle={() => setIsTerminalOpen(!isTerminalOpen)}
      >
        {/* Every surface declares its provenance here, in exactly one place, so
            no view can silently read as live. See lib/provenance.ts. */}
        <ProvenanceFrame
          provenance={provenanceForTab(activeTab, backendHydrated, seatDiscovery)}
          onNavigateToLive={
            activeTab === 'live-seats'
              ? undefined
              : () => {
                  setActiveTab('live-seats');
                  setSelectedObject(null);
                  setActiveForm('none');
                }
          }
        >
          {renderMainView()}
        </ProvenanceFrame>
      </AppShell>

      <InitDialog
        isOpen={isInitOpen}
        onClose={() => setIsInitOpen(false)}
        onInitialize={() => {
          setActiveProject('p-1');
          setIsInitOpen(false);
        }}
      />
      <AddSeatDialog isOpen={activeForm === 'addSeat'} onClose={() => setActiveForm('none')} />
      <ShortcutHelpDialog isOpen={showHelp} onClose={() => setShowHelp(false)} />
      <SupervisorCommandBar
        isOpen={showCommandBar}
        onClose={() => setShowCommandBar(false)}
        onConfirmProposal={handleConfirmProposal}
      />

      {/* Supervisor IM (V2 floating panel) — embedded by default; ⌘K opens;
          Detach opens it in its own OS-level Tauri window (R3). Note: the V2
          SupervisorPanel UI is the canonical IM; we do NOT replace it. */}
      {showSupervisorIM && !supervisorDetached && (
        <IMErrorBoundary>
          <SupervisorPanel onClose={() => setShowSupervisorIM(false)} />
          {/* Detach affordance — rendered alongside SupervisorPanel so the V2
              component itself stays untouched. Fixed top-right of viewport. */}
          <button
            onClick={async () => {
              if (!isTauri()) {
                setSupervisorDetached(true);
                setShowSupervisorIM(false);
                return;
              }
              try {
                await api.openSupervisorWindow();
              } catch (e) {
                console.error('openSupervisorWindow failed', e);
              }
            }}
            title="Open Supervisor in its own window"
            style={{
              position: 'fixed',
              top: 12,
              right: 18,
              zIndex: 10000,
              padding: '4px 12px',
              fontSize: 12,
              fontWeight: 600,
              background: 'var(--sl-surface)',
              color: 'var(--sl-text-primary)',
              border: '1px solid var(--sl-border)',
              borderRadius: 6,
              cursor: 'pointer',
              boxShadow: '0 2px 6px rgba(0,0,0,0.08)',
            }}
          >
            ↗ Detach Supervisor
          </button>
        </IMErrorBoundary>
      )}

      {protectionType && (
        <SwitchProtectionDialog
          isOpen={!!protectionType}
          type={protectionType}
          onConfirm={() => performSwitch(pendingProjectId!)}
          onCancel={() => {
            setPendingProjectId(null);
            setProtectionType(null);
          }}
          onClose={() => setProtectionType(null)}
        />
      )}

      {showReconcileSummary && activeTab !== 'all-projects' && (
        <ReconcileNotification
          count={reconcileIssueCount || 0}
          onClose={() => setShowReconcileSummary(false)}
          onViewInbox={() => {
            setActiveTab('inbox');
            setShowReconcileSummary(false);
          }}
        />
      )}
    </>
  );
};

export default App;
