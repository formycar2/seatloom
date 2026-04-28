import React, { useEffect, useMemo, useState } from 'react';
import { Target } from 'lucide-react';
import AppShell from './layouts/AppShell';
import InboxView from './views/InboxView';
import TimelineView from './views/TimelineView';
import WorkItemsView from './views/WorkItemsView';
import AllProjectsView from './views/AllProjectsView';
import DetailPane from './layouts/DetailPane';
import SeatDetail from './components/SeatDetail';
import SessionDetail from './components/SessionDetail';
import WorkItemDetail from './components/WorkItemDetail';
import HandoffDetail from './components/HandoffDetail';
import WorkItemForm from './components/WorkItemForm';
import HandoffForm from './components/HandoffForm';
import AddSeatDialog from './components/AddSeatDialog';
import InitDialog from './components/InitDialog';
import TerminalPanel from './components/TerminalPanel';
import PipelineProgress from './components/PipelineProgress';
import ReconcileNotification from './components/ReconcileNotification';
import SwitchProtectionDialog from './components/SwitchProtectionDialog';
import { useDataStore } from './stores/useDataStore';
import { useAppStore } from './stores/useAppStore';
import { useGlobalShortcuts } from './hooks/useGlobalShortcuts';
import { getObjectTypeLabel } from './utils/display';

type Tab = 'inbox' | 'timeline' | 'workitems' | 'all-projects';
type FormView = 'none' | 'createWorkItem' | 'createHandoff' | 'pipelineProgress' | 'addSeat';
type SelectedObject = { type: 'Seat' | 'Session' | 'WorkItem' | 'Handoff'; id: string } | null;

const App: React.FC = () => {
  const { projectData, activeProjectId, setActiveProject } = useDataStore();
  const { projectUIStates, updateProjectUIState } = useAppStore();

  const [activeTab, setActiveTab] = useState<Tab>('inbox');
  const [selectedObject, setSelectedObject] = useState<SelectedObject>(null);
  const [activeForm, setActiveForm] = useState<FormView>('none');
  const [isInitOpen, setIsInitOpen] = useState(false);
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [showReconcileSummary, setShowReconcileSummary] = useState(false);
  const [reconcileIssueCount] = useState(0);

  const [timelineFilters, setTimelineFilters] = useState<{
    seatId: string | null;
    workItemId: string | null;
    eventType: string | null;
    timeRange: string;
  }>({ seatId: null, workItemId: null, eventType: null, timeRange: '24h' });

  const [pendingProjectId, setPendingProjectId] = useState<string | null>(null);
  const [protectionType, setProtectionType] = useState<'session' | 'form' | null>(null);

  useGlobalShortcuts(
    () => {
      setSelectedObject(null);
      setActiveForm('none');
    },
    () => {
      setActiveTab(activeTab === 'all-projects' ? 'inbox' : 'all-projects');
    },
    () => {
      setIsTerminalOpen((prev) => !prev);
    },
  );

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects') {
      const savedState = projectUIStates[activeProjectId];
      if (savedState) {
        setActiveTab(savedState.activeTab as Tab);
        setSelectedObject(savedState.selectedObjectId ? ({ type: 'WorkItem', id: savedState.selectedObjectId } as any) : null);
      }
    }
  }, [activeProjectId, projectUIStates]);

  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects' && activeTab !== 'all-projects') {
      updateProjectUIState(activeProjectId, {
        activeTab,
        selectedObjectId: selectedObject?.id || null,
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

    if (type === 'Seat') {
      setTimelineFilters((prev) => ({ ...prev, seatId: id }));
      setActiveTab('timeline');
    } else if (type === 'WorkItem') {
      setTimelineFilters((prev) => ({ ...prev, workItemId: id }));
      setActiveTab('timeline');
    }
  };

  const handleInboxSelect = (item: any) => {
    const current = activeProjectId ? projectData[activeProjectId] : null;
    if (!current) return;

    const ref = item.object_ref?.toLowerCase() || '';
    if (item.type.includes('交接') || item.type.includes('Handoff') || ref.startsWith('ho-')) {
      const handoff = current.handoffs.find((entry: any) => entry.id.toLowerCase() === ref || entry.workitem_id.toLowerCase() === ref);
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
        return;
      }
    }

    setActiveForm('none');
  };

  const currentProjectData = activeProjectId ? projectData[activeProjectId] : null;

  const renderMainView = () => {
    if (activeTab === 'all-projects') {
      return <AllProjectsView onSelectProject={handleProjectSwitchRequest} />;
    }

    switch (activeTab) {
      case 'inbox':
        return <InboxView onSelectObject={handleInboxSelect} />;
      case 'timeline':
        return <TimelineView onSelectObject={handleSelectObject} filters={timelineFilters} onFiltersChange={setTimelineFilters} />;
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
        return <InboxView onSelectObject={handleInboxSelect} />;
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
    if (!selectedObject || !resolvedData) return '详情';

    if (selectedObject.type === 'Seat' && 'name' in resolvedData) return resolvedData.name;
    if (selectedObject.type === 'Session' && 'id' in resolvedData) return `${resolvedData.id} 会话详情`;
    if (selectedObject.type === 'WorkItem' && 'title' in resolvedData) return resolvedData.title;
    if (selectedObject.type === 'Handoff' && 'id' in resolvedData) return `${resolvedData.id} 交接单`;
    return selectedObject.id;
  }, [activeForm, selectedObject, resolvedData]);

  const renderDetailContent = () => {
    if (activeForm === 'createWorkItem') return <WorkItemForm onClose={() => setActiveForm('none')} />;
    if (activeForm === 'createHandoff') {
      return <HandoffForm onClose={() => setActiveForm('none')} workItemId={selectedObject?.type === 'WorkItem' ? selectedObject.id : undefined} />;
    }
    if (activeForm === 'pipelineProgress') return <div className="space-y-4 p-4"><PipelineProgress /></div>;

    if (!selectedObject || !resolvedData) {
      return (
        <div className="h-full flex flex-col items-center justify-center p-8 text-center space-y-4 opacity-50 text-muted-foreground">
          <div className="w-16 h-16 rounded-full border-2 border-dashed border-muted-foreground flex items-center justify-center text-muted-foreground">
            <Target size={24} />
          </div>
          <div className="space-y-2">
            <p className="text-sm font-bold tracking-wide text-foreground/80">请选择左侧对象查看真实状态</p>
            <p className="text-xs leading-6 max-w-[220px]">
              可查看席位、会话、工作项与交接单的当前状态、上下游依赖和最近活动记录。
            </p>
          </div>
        </div>
      );
    }

    switch (selectedObject.type) {
      case 'Seat':
        return <SeatDetail seat={resolvedData as any} />;
      case 'Session':
        return <SessionDetail session={resolvedData as any} />;
      case 'WorkItem':
        return <WorkItemDetail workItem={resolvedData as any} />;
      case 'Handoff':
        return <HandoffDetail handoff={resolvedData as any} />;
      default:
        return null;
    }
  };

  const renderDetailActions = () => {
    if (activeForm !== 'none' && activeForm !== 'pipelineProgress') return null;
    if (!selectedObject && activeForm !== 'pipelineProgress') return null;

    if (activeForm === 'pipelineProgress') {
      return (
        <button
          onClick={() => setActiveForm('none')}
          className="col-span-2 px-3 py-2 bg-destructive text-destructive-foreground rounded text-[10px] font-black tracking-widest hover:opacity-90 transition-all focus:outline-none focus:ring-1 focus:ring-primary"
          tabIndex={0}
        >
          中止本次流水线
        </button>
      );
    }

    if (selectedObject?.type === 'WorkItem') {
      return (
        <>
          <button
            onClick={() => setActiveForm('createHandoff')}
            className="px-3 py-2 bg-primary text-primary-foreground rounded text-[10px] font-black tracking-widest hover:opacity-90 transition-all focus:outline-none focus:ring-1 focus:ring-primary"
            tabIndex={0}
          >
            发起交接
          </button>
          <button
            onClick={() => setActiveForm('pipelineProgress')}
            className="px-3 py-2 bg-secondary border border-border text-foreground rounded text-[10px] font-black tracking-widest hover:bg-accent transition-all focus:outline-none focus:ring-1 focus:ring-primary"
            tabIndex={0}
          >
            运行流水线
          </button>
        </>
      );
    }

    return (
      <button
        onClick={() => setSelectedObject(null)}
        className="col-span-2 px-3 py-2 bg-secondary border border-border rounded text-[10px] font-black tracking-widest hover:bg-black/5 transition-all focus:outline-none focus:ring-1 focus:ring-primary"
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
        }}
        onSelectProject={handleProjectSwitchRequest}
        activeObjectId={selectedObject?.id || null}
        onSelectObject={handleSelectObject}
        onInitProject={() => setIsInitOpen(true)}
        isTerminalOpen={isTerminalOpen}
        onTerminalToggle={() => setIsTerminalOpen((prev) => !prev)}
        terminalPane={<TerminalPanel onClose={() => setIsTerminalOpen(false)} />}
        detailPane={
          (selectedObject || (activeForm !== 'none' && activeForm !== 'addSeat')) && activeTab !== 'all-projects' && (
            <DetailPane
              title={detailTitle}
              type={activeForm === 'pipelineProgress' ? 'Status' : activeForm === 'createWorkItem' || activeForm === 'createHandoff' ? 'Create' : selectedObject?.type || 'Create'}
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
          setIsInitOpen(false);
        }}
      />

      <AddSeatDialog isOpen={activeForm === 'addSeat'} onClose={() => setActiveForm('none')} />

      {protectionType && (
        <SwitchProtectionDialog
          isOpen={!!protectionType}
          type={protectionType}
          onConfirm={() => performSwitch(pendingProjectId!)}
          onCancel={() => {
            setPendingProjectId(null);
            setProtectionType(null);
          }}
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
