import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
import EventDetail from './components/EventDetail';
import HydrationFailure from './components/HydrationFailure';
import SupervisorSyncLost from './components/SupervisorSyncLost';
import { SandboxBanner } from './components/SandboxBanner';
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
import { ArtifactSubtype, ArtifactTemplate, CanonicalEvent, SelectedObjectType } from './types';

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
// `event` carries the whole canonical event through for the 'Event' type: a timeline
// row is its own object and there is nothing to look up in the store, so dropping the
// payload here would leave the detail pane with an id and nothing to show.
type SelectedObject = {
  type: SelectedObjectType;
  id: string;
  event?: CanonicalEvent;
} | null;

const App: React.FC = () => {
  const {
    projectData,
    activeProjectId,
    setActiveProject,
    addWorkItem,
    hydrateFromBackend,
    refreshEvents,
    backendHydrated,
    backendHydrateError,
  } = useDataStore();
  // Live seat discovery status (UM0 HOLD-3): the provenance banner must not
  // claim LIVE while tmux discovery is empty or has failed.
  const seatDiscovery = useLiveSessionsStore((s) => s.seatDiscovery);
  const { projectUIStates, updateProjectUIState, setActiveProjectId } = useAppStore();
  // Snapshot the launch record: no startup write may change the restore target.
  const [persistedProjectId] = useState(() => useAppStore.getState().activeProjectId);
  const projectUIStatesRef = useRef(projectUIStates);
  useEffect(() => {
    projectUIStatesRef.current = projectUIStates;
  });

  // Restore only after a successful backend read. Until this phase completes,
  // the persistence effect below is disabled. A deleted/missing project falls
  // back to the first real backend project (or no project for an empty list).
  // Keep a ref to the latest backend project list without retriggering restore.
  const projectDataOrderRef = useRef<string[]>([]);
  useEffect(() => {
    projectDataOrderRef.current = useDataStore.getState().projects.map((p) => p.id);
  });
  const [restoredPosition, setRestoredPosition] = useState(false);
  // The project id whose arrival is the cold-start restore, cleared as soon as
  // the tab effect has seen it. See the assignment in the restore effect below.
  const coldStartSkip = useRef<string | null>(null);
  useEffect(() => {
    if (restoredPosition) return;
    if (!backendHydrated) return; // do not guess before the read lands
    const ids = new Set(projectDataOrderRef.current);
    if (!ids.has(persistedProjectId ?? '')) {
      // The backend has already selected the first real project, or null.
      coldStartSkip.current = useDataStore.getState().activeProjectId;
      if (projectDataOrderRef.current.length === 0) setActiveProjectId(null);
      setRestoredPosition(true);
      return;
    }
    // Mark BEFORE the project changes. `activeProjectId` is derived from the
    // data store, so it changes as soon as setActiveProject runs, and the tab
    // effect re-runs within the same effect flush — this ref is what tells that
    // re-run "this arrival is the cold-start restore, do not apply its saved tab".
    // Setting it after would be a render too late, which is the mechanism Flux
    // named.
    coldStartSkip.current = persistedProjectId as string;
    setActiveProject(persistedProjectId as string);
    setRestoredPosition(true);
  }, [backendHydrated, persistedProjectId, restoredPosition, setActiveProject, setActiveProjectId]);

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
  // An effect's state updates are visible only on the next render. Gate writes
  // on this acknowledgement, not effect order, so the old tab cannot overwrite
  // the destination's saved tab during startup OR a later project switch.
  const [loadedUIProjectId, setLoadedUIProjectId] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(false);
  const [showCommandBar, setShowCommandBar] = useState(false);
  const [showSupervisorIM, setShowSupervisorIM] = useState(
    () => localStorage.getItem('seatloom.supervisor.open') === 'true',
  );
  const [supervisorDetached, setSupervisorDetached] = useState(false);
  // ── One report, three ways it can be incomplete ────────────────────────
  //
  // The supervisor panel's position is learned from ONE query and TWO
  // subscriptions. Any of the three can fail, and the failures are NOT the same
  // thing:
  //
  //   query fails -> the state was NEVER answered. The worse of the two: there is
  //                 no answer to fall back on, and re-reading is what fixes it.
  //                 Shape `never-answered` (red, retry).
  //   sub fails   -> an answer that WAS correct has stopped being maintained.
  //                 Re-subscribing cannot fix a broken event bus. Shape `stale`
  //                 (yellow, no retry).
  //   not Tauri   -> the effect returns on its first line. Not an error.
  //
  // WHY ONE ACCUMULATED OBJECT RATHER THAN A SINGLE SLOT: the first version had
  // the query's .catch and the subscriptions' reporter each writing the same
  // slot, so the last writer won — and the subscription rejections land in a
  // LATER microtask, so they always did. Flux measured the consequence as F2:
  // when the query AND the subscriptions both fail, the worse failure
  // (never-answered, carrying the query's error text and the only useful action)
  // was hidden behind `stale`. The operator was told "you had an answer that
  // went quiet" when the truth was "you never got one", and the error text they
  // are asked to hand over was gone.
  //
  // So both reporters write into ONE accumulated record: the kind is the WORST of
  // what happened, and every reason is kept. Nothing is overwritten, so the worse
  // failure can no longer lose by arriving second.
  const [supervisorSyncLost, setSupervisorSyncLost] = useState<{
    kind: 'never-answered' | 'stale';
    /// Every reason, in the order it was reported. Never collapsed: the query's
    /// error text is the evidence the operator is asked to hand to Aegis, and a
    /// subscription's names which channel broke.
    reasons: string[];
    /// True when a subscription failed too. Only meaningful alongside
    /// `never-answered` — it is what makes "both are down" sayable.
    subscriptionsDown: boolean;
  } | null>(null);

  // Kept in one place so the two reporters cannot disagree about severity: the
  // query failing always outranks a subscription failing, whichever arrives
  // first. Referentially stable (no state in deps), so listing it in the effect's
  // dependency array costs nothing.
  const reportSupervisorFailure = useCallback(
    (failure: { kind: 'never-answered' | 'stale'; reason: string }) => {
      setSupervisorSyncLost((prev) => {
        const reasons = prev ? [...prev.reasons, failure.reason] : [failure.reason];
        // WHAT THIS GUARD IS FOR: severity must be a function of WHAT failed,
        // not of who reported first. A query failure outranks a subscription
        // failure, so a subscription arriving second must not demote a
        // never-answered into a stale. The `failure.kind` disjunct alone cannot
        // do that — it reads the CURRENT failure, which by then is the stale one.
        //
        // WHY IT LOOKED DEAD (2026-10-27, measured by three seats): the app
        // invokes askSupervisorWindowStatus() BEFORE it registers the two
        // subscriptions, so the query's rejection lands FIRST and this disjunct
        // never fires. Flux reported that a control mutation — M2 PLUS moving the
        // query after the two subscriptions — turned RED (2 failures). That
        // number was WRONG: measured against this suite it is 20 passed / 0 red,
        // reproduced independently by Nimbus (three orderings) and by Aegis. What
        // was right was the DIRECTION — that ordering is the one that exposes the
        // guard — and what was missing was a test that reaches it.
        //
        // THAT TEST NOW EXISTS: `a subscription failing AFTER the query does not
        // demote the shape` in App.supervisorSync.test.tsx. It holds the
        // reembedded subscription back (`holdReembedListen`), lets the query fail
        // first, then releases the subscription. Measured, on the guard as it
        // stands: 21 passed. Measured with M2 (`kind = failure.kind`): 1 red —
        // this test. Measured with only the `prev?.kind` disjunct removed: 1 red
        // — the same test. Measured with Flux's ordering mutation (M2 plus the
        // query moved after both subscriptions): 1 red — the same test.
        //
        // So the guard is load-bearing and now COVERED, rather than load-bearing
        // and hidden. Kept for the same reason as before: if anyone reorders this
        // effect, or a third reporter is added, dropping it would silently
        // reintroduce "last writer wins" — the F2 defect this whole shape exists
        // to remove.
        const kind: 'never-answered' | 'stale' =
          prev?.kind === 'never-answered' || failure.kind === 'never-answered'
            ? 'never-answered'
            : 'stale';
        const subscriptionsDown = (prev?.subscriptionsDown ?? false) || failure.kind === 'stale';
        return { kind, reasons, subscriptionsDown };
      });
    },
    [],
  );

  useEffect(() => {
    localStorage.setItem('seatloom.supervisor.open', showSupervisorIM ? 'true' : 'false');
  }, [showSupervisorIM]);

  // Reflect the current detached/embedded state from the backend on mount so
  // reopening the main window after detaching the IM doesn't re-show the
  // embedded panel next to the detached window.
  //
  // ── Three ways this state can fail, and why they are not one thing ─────────
  //
  // 1. THE STARTUP QUERY FAILS. `supervisorDetached` then keeps its `useState`
  //    initial `false` — and that value is not neutral, it is the claim "the
  //    panel is in this window". This is the FIRST read, so the state was never
  //    answered at all. Reported as `never-answered` (red, with a retry):
  //    re-reading is what fixes a single failed query.
  //
  //    This used to be `.catch(() => {})`, which is the exact shape this whole
  //    design exists to remove — Flux found it as F1 while verifying the
  //    subscription half, and measured it (`shownWhenQueryFails=false,
  //    shownWhenSubFails=true`) rather than inferring it.
  //
  // 2. A SUBSCRIPTION FAILS. An answer that WAS correct has stopped being
  //    maintained. Reported as `stale` (yellow, no retry): re-subscribing
  //    against a broken event bus produces noise and no state.
  //
  // 3. NOT Tauri at all. The effect returns on its first line, so nothing here
  //    runs and nothing is reported. A browser dev session is not an error.
  //
  // Ask the desktop where the supervisor panel is. Lives at component scope, not
  // inside the effect, because the never-answered strip's retry button calls it
  // too — a button that could only re-run the mount effect would not be a retry.
  //
  // `cancelled` is per effect run, so this deliberately does NOT check it: the
  // retry is an explicit operator action taken while mounted, and a read that
  // lands after an unmount would only set state on an unmounted component.
  const askSupervisorWindowStatus = useCallback(() => {
    api
      .supervisorWindowStatus()
      .then((s) => {
        setSupervisorDetached(s.detached);
        // A successful read clears the QUERY's own failure: the retry button is
        // only honest if succeeding actually removes it.
        //
        // It clears only the query's failure. A broken subscription is a separate
        // fact the operator still needs to know — wiping it here would make the
        // retry hide a live problem, which is the mirror image of F2 (there: a
        // failure hidden by arriving second; here: a failure hidden by an
        // unrelated success). So the shape demotes to stale with the
        // subscription reasons kept.
        setSupervisorSyncLost((prev) => {
          if (!prev || prev.kind !== 'never-answered') return prev;
          if (!prev.subscriptionsDown) return null;
          return {
            kind: 'stale',
            reasons: prev.reasons.filter((r) => !r.startsWith('查询监督面板位置失败')),
            subscriptionsDown: true,
          };
        });
      })
      .catch((err) => {
        reportSupervisorFailure({
          kind: 'never-answered',
          reason: `查询监督面板位置失败:${err instanceof Error ? err.message : String(err)}`,
        });
      });
  }, [reportSupervisorFailure]);

  useEffect(() => {
    if (!isTauri()) return;
    let cancelled = false;

    // Set when a subscription fails. The startup query above still answers the
    // state; this records that the answer has stopped being maintained.
    //
    // Both channels are subscribed in parallel, so both can fail in the same
    // tick. The first version kept only the last message and silently dropped
    // the other — a reader could not tell that BOTH links were down, which is
    // the difference between "one channel is flaky" and "the event bus is not
    // accepting subscriptions at all". Each channel keeps its OWN reason, and
    // both are shown, because the two refusals are not necessarily the same
    // failure and collapsing them would invent a single cause.
    const lostChannels = new Map<string, string>();
    const reportSubscriptionFailure = (eventName: string) => (err: unknown) => {
      if (cancelled) return;
      lostChannels.set(eventName, err instanceof Error ? err.message : String(err));
      const lines = [...lostChannels].map(([name, detail]) => `订阅 ${name} 失败:${detail}`);
      if (lostChannels.size > 1) lines.push('(两条通道都断了)');
      reportSupervisorFailure({ kind: 'stale', reason: lines.join('; ') });
    };

    askSupervisorWindowStatus();

    let offDetached: (() => void) | null = null;
    let offReembed: (() => void) | null = null;
    onSupervisorDetached(() => {
      setSupervisorDetached(true);
      setShowSupervisorIM(false);
    })
      .then((fn) => {
        if (cancelled) {
          // The component went away while the subscription was still being set
          // up; the listener was registered, so it must be removed.
          fn();
          return;
        }
        offDetached = fn;
      })
      .catch(reportSubscriptionFailure('supervisor:detached'));
    onSupervisorReembedded(() => {
      setSupervisorDetached(false);
    })
      .then((fn) => {
        if (cancelled) {
          fn();
          return;
        }
        offReembed = fn;
      })
      .catch(reportSubscriptionFailure('supervisor:reembedded'));
    return () => {
      cancelled = true;
      if (offDetached) offDetached();
      if (offReembed) offReembed();
    };
    // Both callbacks are useCallback with empty (or all-stable) dependency lists,
    // so they are referentially stable for the life of the component. Listing
    // them is therefore correct AND free: the effect still runs exactly once,
    // because neither ever changes identity. (It must stay mount-only —
    // re-running would double-register the two subscriptions.)
  }, [askSupervisorWindowStatus, reportSupervisorFailure]);

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
    if (
      restoredPosition &&
      !activeProjectId &&
      activeTab !== 'all-projects' &&
      activeTab !== 'live-seats'
    ) {
      setIsInitOpen(true);
    }
  }, [restoredPosition, activeProjectId, activeTab]);

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

  //
  // A COLD START begins on 派单. The per-project tab memory is a real feature and
  // is NOT removed — it still applies when the operator switches projects inside a
  // session. What it must not do is overwrite the first screen of a fresh launch:
  // `useState<Tab>('dispatches')` above is the contract, and this effect used to
  // beat it on mount because the store is persisted without a partialize, so
  // `projectUIStates` survives a restart.
  //
  // Measured on the operator's machine (2026-10-29): the persisted project was
  // `project_delivery/QL` with `activeTab: 'live-seats'`, so the app opened on
  // 实时席位 every launch — and the first-screen card asserted the opposite.
  //
  // THE GATE. `coldStartProjectId` is the project the restore effect lands on,
  // captured as STATE there so ordering between the two effects cannot matter.
  // Skipping only that one project's tab — and only on the run where it first
  // appears — leaves every other behaviour exactly as it was: a switch to a
  // different project still restores that project's tab, and the write effect
  // below is untouched, so the memory is neither lost nor frozen.
  //
  // Two earlier attempts were measured and reverted, recorded so they are not
  // retried: gating the WRITE effect froze the remembered project permanently;
  // delaying `setLoadedUIProjectId` broke the write gate's own precondition.
  useEffect(() => {
    if (activeProjectId && activeProjectId !== 'all-projects') {
      // The cold-start restore arrives once, on the project the restore effect
      // put there. Its saved tab must NOT be written over the 派单 default — but
      // every later arrival at that project (a switch back) must restore. So the
      // skip is consumed on first sight, not sticky.
      //
      // A ref, not state: state set here would not be visible to the tab effect's
      // re-run within the same effect flush, which is the mechanism Flux named.
      // Before the restore has landed, NO arrival at a project is the operator's:
      // the app is still showing defaults ('p-1') and then whatever the data
      // store picked for itself while the backend read was in flight (measured:
      // the first real project). Applying a saved tab to either would be
      // answering "where was the operator?" with a project they never opened.
      //
      // After it lands, every arrival restores normally — including a switch back
      // to the restored project, because the skip is consumed on first sight.
      const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
      if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;
      if (!skipColdStart) {
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
    }
    setLoadedUIProjectId(activeProjectId);
  }, [activeProjectId, restoredPosition]);

  // Remember which project the operator is in. This is NOT the UI position: it is
  // which project the next launch returns to, and it is written here rather than
  // in `chooseTab` because a project SWITCH must also be remembered, and the
  // switch path does not go through a tab change.
  //
  // Kept separate from the tab persistence on purpose. The ruling removed the
  // effect-driven TAB write; it did not ask for the project id to stop being
  // remembered, and without it a launch could not return to where the operator
  // was.
  useEffect(() => {
    if (!restoredPosition || loadedUIProjectId !== activeProjectId) return;
    if (activeProjectId && activeProjectId !== 'all-projects') {
      setActiveProjectId(activeProjectId);
    }
  }, [restoredPosition, loadedUIProjectId, activeProjectId, setActiveProjectId]);

  // ─── Persisting the UI position ───
  //
  // RULED 2026-10-29 (Lyra): the UI position is persisted from OPERATOR INPUT, not
  // from an effect.
  //
  // The effect-driven write could not tell "restored" from "chosen": both produce
  // the same `activeTab` value, so every gate I tried either left the cold-start
  // default being persisted over the operator's real saved tab (the round-6
  // pollution, which came back and was caught by Lyra's mutation 2), or swallowed
  // the operator's legitimate writes. Five attempts traded one for the other.
  //
  // The information that distinguishes them exists only at the call site, so the
  // write belongs there. ONE wrapper, used by every operator-driven tab change:
  // eight scattered `updateProjectUIState` calls would be eight chances to forget
  // one, and forgetting one is silent position loss — the same class of bug that
  // took ten rounds to find.
  //
  // The restore at the tab effect above deliberately does NOT go through this.
  // It is the only tab change with no operator input, and it must not persist.
  const chooseTab = useCallback(
    (tab: Tab) => {
      setActiveTab(tab);
      if (activeProjectId && activeProjectId !== 'all-projects' && tab !== 'all-projects') {
        updateProjectUIState(activeProjectId, {
          activeTab: tab,
          selectedObjectId: selectedObject?.id || null,
          selectedObjectType: selectedObject?.type || null,
        });
        // Remember which project the operator is in, so the next launch returns here.
        setActiveProjectId(activeProjectId);
      }
    },
    [activeProjectId, selectedObject, updateProjectUIState, setActiveProjectId],
  );

  // ─── Project switching ───
  const handleProjectSwitchRequest = (id: string) => {
    if (id === 'all-projects') {
      chooseTab('all-projects');
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
    if (activeTab === 'all-projects') chooseTab('dashboard');
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
    setSelectedObject({
      type: type as any,
      id,
      ...(type === 'Event' && typeof data !== 'string' ? { event: data } : {}),
    });
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
    chooseTab('workitems');
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
      // A timeline row is its own object: the event arrives whole from the click, so
      // there is nothing to look up in the store. Before 'Event' existed in
      // SelectedObjectType this fell to `default` -> null -> no detail pane -> a click
      // that did nothing.
      case 'Event':
        return selectedObject.event ?? null;
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
      case 'Event':
        // `resolvedData` for this type IS `selectedObject.event` (see resolvedData);
        // read it off the selection so no cast over the union is needed.
        return <EventDetail event={selectedObject.event as CanonicalEvent} />;
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
            chooseTab('inbox');
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

  // 2026-10-16: a hydration failure used to be stored in the store and never rendered,
  // so a failed read looked like "the product has three projects". Shown once, above the
  // shell, on every tab — and the shell below renders no projects at all rather than the
  // three hardcoded seeds it used to fall back to.
  const hydrationFailed = !backendHydrated && backendHydrateError !== null;

  return (
    <>
      {hydrationFailed && (
        <HydrationFailure
          error={backendHydrateError ?? '(没有错误文本 —— 这本身就该当缺陷看)'}
          onRetry={() => void hydrateFromBackend()}
        />
      )}
      {/* Outside the `showSupervisorIM && !supervisorDetached` block on purpose: a
          stale `supervisorDetached === false` is exactly the case where the panel
          is believed embedded, and hiding the warning behind that belief would
          make it invisible precisely when it is needed. */}
      {supervisorSyncLost !== null && (
        <SupervisorSyncLost
          kind={supervisorSyncLost.kind}
          reasons={supervisorSyncLost.reasons}
          subscriptionsDown={supervisorSyncLost.subscriptionsDown}
          onRetry={() => askSupervisorWindowStatus()}
        />
      )}
      <AppShell
        activeTab={activeTab}
        onTabChange={(tab) => {
          chooseTab(tab as Tab);
          setSelectedObject(null);
          setActiveForm('none');
        }}
        onSelectProject={handleProjectSwitchRequest}
        onInitProject={() => setIsInitOpen(true)}
        onShowHelp={() => setShowHelp(true)}
        isTerminalOpen={isTerminalOpen}
        onTerminalToggle={() => setIsTerminalOpen(!isTerminalOpen)}
      >
        {/* 第二层「我在看哪一份构建」。第一层是图标(只解决 Dock 和 Cmd-Tab),
            窗口打开后两份内容一模一样,所以这里常驻一条身份条:只有沙箱会渲染,
            主版什么都不渲染。它读后端命令,不读 localStorage —— 那三个无命名空间的
            键两个版本同时开会互抢。它没有关闭按钮:这是身份,不是提示。
            详见 components/SandboxBanner.tsx。 */}
        <SandboxBanner />

        {/* Every surface declares its provenance here, in exactly one place, so
            no view can silently read as live. See lib/provenance.ts. */}
        <ProvenanceFrame
          provenance={provenanceForTab(activeTab, backendHydrated, seatDiscovery)}
          onNavigateToLive={
            activeTab === 'live-seats'
              ? undefined
              : () => {
                  chooseTab('live-seats');
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
          // Re-read the project list instead of planting a seeded id. There is no
          // seeded project to "initialize" any more — clicking this button used to
          // call setActiveProject('p-1'), i.e. put a test-only project into state
          // where the persistence effect then wrote it to disk as the recorded
          // position. Re-reading is the honest action: either the backend has
          // projects by now and the list lands, or the empty state explains itself.
          void hydrateFromBackend();
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
            chooseTab('inbox');
            setShowReconcileSummary(false);
          }}
        />
      )}
    </>
  );
};

export default App;
