import React, { useEffect, useMemo, useState } from 'react';
import { Settings, Target } from 'lucide-react';
import AppShell from './layouts/AppShell';
import InboxView from './views/InboxView';
import TimelineView from './views/TimelineView';
import WorkItemsView from './views/WorkItemsView';
import AllProjectsView from './views/AllProjectsView';
import MobileCompanionView from './views/MobileCompanionView';
import DetailPane from './layouts/DetailPane';
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
import TerminalPanel from './components/TerminalPanel';
import PipelineProgress from './components/PipelineProgress';
import ReconcileNotification from './components/ReconcileNotification';
import SwitchProtectionDialog from './components/SwitchProtectionDialog';
import { useDataStore } from './stores/useDataStore';
import { useAppStore } from './stores/useAppStore';
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts';
import { ArtifactSubtype, ArtifactTemplate, SelectedObjectType } from './types';
import { THEME_PRESETS } from './styles/theme';

type Tab = 'inbox' | 'timeline' | 'workitems' | 'all-projects' | 'mobile';
type FormView = 'none' | 'createWorkItem' | 'createHandoff' | 'pipelineProgress' | 'addSeat';
type SelectedObject = { type: SelectedObjectType; id: string } | null;

const App: React.FC = () => {
  const { projectData, activeProjectId, setActiveProject, addWorkItem } = useDataStore();
  const { projectUIStates, updateProjectUIState, themePreset, setThemePreset } = useAppStore();

  const [activeTab, setActiveTab] = useState<Tab>('inbox');
  const [selectedObject, setSelectedObject] = useState<SelectedObject>(null);
  const [showConfigMenu, setShowConfigMenu] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [showCommandBar, setShowCommandBar] = useState(false);

  useEffect(() => {
    // Sync theme on mount and change
    document.body.setAttribute('data-theme', themePreset);
  }, [themePreset]);

  const [activeForm, setActiveForm] = useState<FormView>('none');
  const [isInitOpen, setIsInitOpen] = useState(false);
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [showReconcileSummary, setShowReconcileSummary] = useState(false);
  const [reconcileIssueCount] = useState(0);

  const [timelineFilters, setTimelineFilters] = useState<{
    seatId: string | null;
    workItemId: string | null;
    eventType: string | null;
    artifactTemplate: ArtifactTemplate | null;
    artifactSubtype: ArtifactSubtype | null;
    timeRange: string;
  }>({ seatId: null, workItemId: null, eventType: null, artifactTemplate: null, artifactSubtype: null, timeRange: '24h' });

  const [pendingProjectId, setPendingProjectId] = useState<string | null>(null);
  const [protectionType, setProtectionType] = useState<'session' | 'form' | null>(null);

  useEffect(() => {
    // Automatically trigger initialization if no project is active
    if (!activeProjectId && activeTab !== 'all-projects') {
      setIsInitOpen(true);
    }
  }, [activeProjectId, activeTab]);

  useGlobalShortcuts(
    () => {
      setSelectedObject(null);
      setActiveForm('none');
      setShowCommandBar(false);
    },
    () => {
      // Re-routing Cmd+K from this to CommandBar, but TopNav still handles manual switch.
      // This parameter in useGlobalShortcuts was previously for Tab toggle.
      // In the new hook signature, this is onToggleTab, which I will repurpose or leave.
      // Actually, S7B says Cmd+K should open the bar. 
      // The hook now calls onOpenCommandBar for Cmd+K.
      // So this callback (onToggleTab) is now effectively unused or can be triggered by another key if desired.
    },
    () => {
      setIsTerminalOpen((prev) => !prev);
    },
    () => {
      setShowHelp(true);
    },
    () => {
      setShowCommandBar(true);
    }
  );

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects') {
      const savedState = projectUIStates[activeProjectId];
      if (savedState) {
        setActiveTab(savedState.activeTab as Tab);
        setSelectedObject(
          savedState.selectedObjectId && savedState.selectedObjectType
            ? { type: savedState.selectedObjectType, id: savedState.selectedObjectId }
            : null,
        );
      }
    }
  }, [activeProjectId, projectUIStates]);

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects' && activeTab !== 'all-projects') {
      updateProjectUIState(activeProjectId, {
        activeTab,
        selectedObjectId: selectedObject?.id || null,
        selectedObjectType: selectedObject?.type || null,
      });
    }
  }, [activeTab, selectedObject, activeProjectId, updateProjectUIState]);

  const handleProjectSwitchRequest = (id: string) => {
    if (id === 'all-projects') {
      setActiveTab('all-projects');
      return;
    }

    const currentData = activeProjectId ? projectData[activeProjectId] : null;
    const hasRunningSessions = currentData?.sessions.some((session) => session.status === 'Running' || session.status === 'InputRequired');
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
    if (activeTab === 'all-projects') setActiveTab('inbox');
  };

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

    // Auto-open terminal for running or blocked sessions
    if (type === 'Session') {
      const session = typeof data === 'string' 
        ? currentProjectData?.sessions.find(s => s.id === data) 
        : data;
      
      if (session && (session.status === 'Running' || session.status === 'InputRequired')) {
        setIsTerminalOpen(true);
      }
    }

    if (type === 'Seat') {
      setTimelineFilters((prev) => ({ ...prev, seatId: id }));
      setActiveTab('timeline');
    } else if (type === 'WorkItem') {
      setTimelineFilters((prev) => ({ ...prev, workItemId: id }));
      setActiveTab('timeline');
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
    const id = ref.split('-')[1] || ref;

    if (ref.startsWith('ho-')) {
      const handoff = current.handoffs.find((entry: any) => entry.id.toLowerCase() === ref);
      if (handoff) {
        setSelectedObject({ type: 'Handoff', id: handoff.id });
        setActiveForm('none');
        return;
      }
    }

    if (ref.startsWith('wi-')) {
      const workItem = current.workItems.find((entry: any) => entry.id.toLowerCase() === ref);
      if (workItem) {
        setSelectedObject({ type: 'WorkItem', id: workItem.id });
        setActiveForm('none');
        return;
      }
    }

    if (ref.startsWith('ses-')) {
      const session = current.sessions.find((entry: any) => entry.id.toLowerCase() === ref);
      if (session) {
        setSelectedObject({ type: 'Session', id: session.id });
        setActiveForm('none');
        
        // Auto-open terminal for running or blocked sessions from Inbox
        if (session.status === 'Running' || session.status === 'InputRequired') {
          setIsTerminalOpen(true);
        }
        return;
      }
    }

    if (ref.startsWith('ar-')) {
      const artifact = current.artifacts.find((entry: any) => entry.id.toLowerCase() === ref);
      if (artifact) {
        setSelectedObject({ type: 'Artifact', id: artifact.id });
        setActiveForm('none');
        return;
      }
    }

    const linkedArtifactId = item.linked_artifact_ids?.find((artifactId: string) =>
      current.artifacts.some((artifact: any) => artifact.id === artifactId),
    );

    if (linkedArtifactId) {
      setSelectedObject({ type: 'Artifact', id: linkedArtifactId });
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

  const currentProjectData = activeProjectId ? projectData[activeProjectId] : null;

  const renderMainView = () => {
    if (activeTab === 'all-projects') {
      return <AllProjectsView onSelectProject={handleProjectSwitchRequest} />;
    }

    if (activeTab === 'mobile') {
      return <MobileCompanionView onSelectObject={handleInboxSelect} />;
    }

    switch (activeTab) {
      case 'inbox':
        return <InboxView onSelectObject={handleInboxSelect} selectedObjectId={selectedObject?.id} />;
      case 'timeline':
        return (
          <TimelineView 
            onSelectObject={handleSelectObject} 
            onOpenArtifact={handleOpenArtifact}
            filters={timelineFilters} 
            onFiltersChange={setTimelineFilters} 
          />
        );
      case 'workitems':
        return (
          <WorkItemsView
            onSelectWI={(workItem) =>
              workItem === ('Action:AddWorkItem' as any)
                ? handleSelectObject('Action', 'AddWorkItem')
                : handleSelectObject('WorkItem', workItem)
            }
          />
        );
      default:
        return <InboxView onSelectObject={handleInboxSelect} selectedObjectId={selectedObject?.id} />;
    }
  };

  const resolvedData = useMemo(() => {
    if (!selectedObject || !currentProjectData) return null;

    switch (selectedObject.type) {
      case 'Seat':
        return currentProjectData.seats.find((seat) => seat.id === selectedObject.id) ?? null;
      case 'Session':
        return currentProjectData.sessions.find((session) => session.id === selectedObject.id) ?? null;
      case 'WorkItem':
        return currentProjectData.workItems.find((workItem) => workItem.id === selectedObject.id) ?? null;
      case 'Artifact':
        return currentProjectData.artifacts.find((artifact) => artifact.id === selectedObject.id) ?? null;
      case 'Handoff':
        return currentProjectData.handoffs.find((handoff) => handoff.id === selectedObject.id) ?? null;
      default:
        return null;
    }
  }, [selectedObject, currentProjectData]);

  const detailTitle = useMemo(() => {
    if (activeForm === 'pipelineProgress') return '流水线执行状态';
    if (activeForm === 'createWorkItem') return '新建工作项';
    if (activeForm === 'createHandoff') return '发起交接';
    if (!selectedObject || !resolvedData) return '项目概览 (OVERVIEW)';

    if (selectedObject.type === 'Seat' && 'name' in resolvedData) return resolvedData.name;
    if (selectedObject.type === 'Session' && 'id' in resolvedData) return `${resolvedData.id} 会话详情`;
    if (selectedObject.type === 'WorkItem' && 'title' in resolvedData) return resolvedData.title;
    if (selectedObject.type === 'Artifact' && 'title' in resolvedData) return resolvedData.title;
    if (selectedObject.type === 'Handoff' && 'id' in resolvedData) return `${resolvedData.id} 交接单`;
    return selectedObject.id;
  }, [activeForm, selectedObject, resolvedData]);

  const renderDetailContent = () => {
    if (activeForm === 'pipelineProgress') return <PipelineProgress />;
    if (activeForm === 'createWorkItem') return <WorkItemForm onClose={() => setActiveForm('none')} />;
    if (activeForm === 'createHandoff') return <HandoffForm onClose={() => setActiveForm('none')} workItemId={selectedObject?.type === 'WorkItem' ? selectedObject.id : undefined} />;
    
    if (!selectedObject || !resolvedData) {
      return <ProjectOverview />;
    }

    switch (selectedObject.type) {
      case 'Seat':
        return <SeatDetail seat={resolvedData as any} />;
      case 'Session':
        return <SessionDetail session={resolvedData as any} onOpenArtifact={handleOpenArtifact} />;
      case 'WorkItem':
        return <WorkItemDetail workItem={resolvedData as any} onOpenArtifact={handleOpenArtifact} />;
      case 'Artifact':
        return <ArtifactDetail artifact={resolvedData as any} />;
      case 'Handoff':
        return <HandoffDetail handoff={resolvedData as any} onOpenArtifact={handleOpenArtifact} />;
      default:
        return null;
    }
  };

  const renderDetailActions = () => {
    if (activeForm !== 'none' && activeForm !== 'addSeat') return null;
    if (!selectedObject) return null;

    if (selectedObject.type === 'WorkItem') {
      return (
        <button
          onClick={() => setActiveForm('createHandoff')}
          className="col-span-2 px-3 py-2 bg-primary text-surface rounded text-[10px] font-black tracking-widest hover:opacity-90 transition-all shadow-lg shadow-primary/20"
        >
          发起交接 (HANDOFF)
        </button>
      );
    }

    return (
      <button
        onClick={() => setSelectedObject(null)}
        className="col-span-2 px-3 py-2 bg-secondary border border-border rounded text-[10px] font-black tracking-widest hover:bg-accent transition-all focus:outline-none focus:ring-1 focus:ring-primary"
        tabIndex={0}
      >
        收起详情
      </button>
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
        activeObjectId={selectedObject?.id || null}
        onSelectObject={handleSelectObject}
        onSelectProject={handleProjectSwitchRequest}
        onInitProject={() => setIsInitOpen(true)}
        onShowHelp={() => setShowHelp(true)}
        isTerminalOpen={isTerminalOpen}
        onTerminalToggle={() => setIsTerminalOpen(!isTerminalOpen)}
        hideSidebar={activeTab === 'mobile'}
        detailPane={
          activeTab !== 'all-projects' && activeTab !== 'mobile' && (
            <DetailPane
              title={detailTitle}
              type={activeForm === 'pipelineProgress' ? 'Status' : activeForm === 'createWorkItem' || activeForm === 'createHandoff' ? 'Create' : selectedObject?.type || 'Overview'}
              id={activeForm === 'pipelineProgress' ? 'RUN-085' : selectedObject?.id || ''}
              onClose={() => {
                setSelectedObject(null);
                setActiveForm('none');
              }}
              actions={renderDetailActions()}
            >
              {renderDetailContent()}
            </DetailPane>
          )
        }
      >
        {renderMainView()}
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

      {/* Floating System Config */}
      <div className="fixed bottom-12 right-6 z-50 flex flex-col items-end gap-3">
        {showConfigMenu && (
          <div className="w-56 bg-card border border-border rounded-2xl shadow-2xl overflow-hidden animate-in slide-in-from-bottom-2 duration-200">
            <div className="bg-primary/5 px-4 py-3 border-b border-border">
              <span className="text-[10px] font-black text-ink-soft uppercase tracking-widest">系统配置 (CONFIG)</span>
            </div>
            <div className="p-2 space-y-3">
              <div className="space-y-1.5">
                <span className="text-[9px] font-black text-ink-faint uppercase tracking-widest px-2">主题预设</span>
                <div className="flex flex-col gap-0.5">
                  {THEME_PRESETS.map((p) => (
                    <button
                      key={p.id}
                      onClick={() => setThemePreset(p.id)}
                      className={`flex items-center justify-between px-3 py-2 rounded-xl transition-all ${
                        themePreset === p.id 
                          ? 'bg-primary text-surface font-bold shadow-lg shadow-primary/20' 
                          : 'hover:bg-primary/5 text-ink-soft'
                      }`}
                    >
                      <span className="text-[11px]">{p.label}</span>
                      <div className={`w-1.5 h-1.5 rounded-full ${themePreset === p.id ? 'bg-surface' : 'bg-primary/20'}`} />
                    </button>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}
        <button
          onClick={() => setShowConfigMenu(!showConfigMenu)}
          className={`w-10 h-10 rounded-full flex items-center justify-center transition-all shadow-xl ${
            showConfigMenu 
              ? 'bg-primary text-surface rotate-90 shadow-primary/20' 
              : 'bg-card text-ink-soft border border-border hover:text-primary hover:border-primary/40'
          }`}
          title="系统配置"
        >
          <Settings size={20} />
        </button>
      </div>
    </>
  );
};

export default App;
