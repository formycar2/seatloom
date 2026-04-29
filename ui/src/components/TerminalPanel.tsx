import React, { useState } from 'react';
import {
  ChevronRight,
  Circle,
  Link,
  Play,
  Plus,
  Terminal,
  X,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';
import { useLocaleStore } from '../stores/useLocaleStore';
import { getRuntimeLabel, getSessionStatusLabel, formatTimeZh } from '../utils/display';
import WrapLaunchDialog from './WrapLaunchDialog';
import AttachSessionDialog from './AttachSessionDialog';

interface TerminalPanelProps {
  onClose: () => void;
}

const TerminalPanel: React.FC<TerminalPanelProps> = ({ onClose }) => {
  const { t } = useLocaleStore();
  const { activeProjectId, projectData } = useDataStore();
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [showWrapLaunch, setShowWrapLaunch] = useState(false);
  const [showAttach, setShowAttach] = useState(false);

  const currentData = activeProjectId ? projectData[activeProjectId] : null;
  const activeSessions = (currentData?.sessions || []).filter(
    (s) => s.status === 'Running' || s.status === 'InputRequired'
  );

  const activeSession =
    activeSessions.find((s) => s.id === activeSessionId) || activeSessions[0] || null;
  const activeSeat = activeSession
    ? currentData?.seats.find((s) => s.id === activeSession.seat_id)
    : null;

  const relatedWorkItems = activeSession
    ? (currentData?.workItems || []).filter((wi) => wi.owner_seat_id === activeSession.seat_id)
    : [];

  const terminalFeeds: Record<string, string[]> = {
    'ses-401': [
      '[Codex] Indexed 124 files in seatloom...',
      '[Codex] Found 3 coordination drift points.',
      '[Codex] Waiting for next instruction.',
    ],
    'ses-403': [
      '[OpenCode] Running cd ui && pnpm build...',
      '[OpenCode] Build PASS (1.5s)',
      '[OpenCode] Syncing MIRA-2026-04-28-serial-restart-s5a-handoff-state-strip-v1.md',
    ],
    'ses-406': [
      '[Codex] Updating mockData.ts with high-density ZH content...',
      '[Codex] Rewriting TimelineView.tsx filtering logic...',
      '[Codex] 4 files modified, pending verification.',
    ],
  };

  const feed = activeSession ? terminalFeeds[activeSession.id] || ['[实时面板] 当前会话暂无额外日志，等待新的协调事件写入。'] : [];

  return (
    <div className="flex-1 flex flex-col bg-[var(--terminal-preview)] text-gray-300 font-mono text-xs overflow-hidden animate-in fade-in duration-700 border-t border-white/5">
      <div className="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-black/20">
        <div className="flex items-center gap-6 min-w-0 flex-1">
          <h2 className="text-sm font-black tracking-[0.24em] text-white/90 flex items-center gap-2 whitespace-nowrap">
            <Terminal size={18} className="text-primary" />
            {t.terminal.title}
          </h2>
          <div className="flex gap-2 items-center min-w-0 overflow-x-auto pb-1">
            {activeSessions.map((session) => {
              const seat = currentData?.seats.find((item) => item.id === session.seat_id);
              return (
                <button
                  key={session.id}
                  onClick={() => setActiveSessionId(session.id)}
                  className={`px-3 py-1 rounded border transition-all whitespace-nowrap ${
                    activeSession?.id === session.id
                      ? 'bg-primary/20 text-primary border-primary/50 font-bold'
                      : 'border-white/10 hover:bg-white/5 text-gray-400'
                  }`}
                >
                  {seat?.name || session.seat_id} / {session.id}
                  {session.status === 'InputRequired' && <span className="ml-2 inline-block w-1.5 h-1.5 bg-status-warning rounded-full animate-pulse" />}
                </button>
              );
            })}

            <div className="h-6 w-px bg-white/10 mx-2" />

            <button onClick={() => setShowWrapLaunch(true)} className="p-1 hover:bg-white/10 text-gray-400 hover:text-white rounded transition-colors" title="包裹式启动新会话">
              <Plus size={16} />
            </button>
            <button onClick={() => setShowAttach(true)} className="p-1 hover:bg-white/10 text-gray-400 hover:text-white rounded transition-colors" title="接入已有运行进程">
              <Link size={16} />
            </button>
          </div>
        </div>
        <div className="flex items-center gap-3 text-[10px] font-bold text-gray-500">
          {activeSession ? (
            <span className="flex items-center gap-1.5 font-black tracking-widest text-status-active"><Circle size={8} className="fill-current" /> 实时同步中</span>
          ) : (
            <span className="flex items-center gap-1.5 font-black tracking-widest opacity-40"><Circle size={8} className="fill-current" /> 空闲</span>
          )}
          <span className="px-2 py-0.5 bg-white/10 rounded">v1.14.28</span>
          <button onClick={onClose} className="p-1 hover:bg-white/10 hover:text-white rounded transition-colors" title="收起终端面板">
            <X size={14} />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-8 leading-relaxed max-w-5xl mx-auto w-full">
        {activeSession ? (
          <div className="space-y-5">
            <div className="flex gap-3 flex-wrap">
              <span className="text-status-active font-bold">{activeSeat?.name || '当前席位'}@seatloom</span>
              <span className="text-primary font-bold">{activeSession.workspace_path}</span>
              <span className="text-white font-black">$</span>
              <span className="text-white italic">聚焦任务 "{relatedWorkItems[0]?.title || '继续处理当前协调任务'}"</span>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-[11px]">
              <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-2">
                <div className="text-gray-500">当前会话概览</div>
                <div className="text-white font-semibold">{getSessionStatusLabel(activeSession.status)} · {getRuntimeLabel(activeSession.runtime)}</div>
                <div className="text-gray-400">分支：{activeSession.branch || '未记录分支'}</div>
                <div className="text-gray-400">启动时间：{formatTimeZh(activeSession.created_at)}{activeSession.pid ? ` · PID ${activeSession.pid}` : ''}</div>
              </div>
              <div className="rounded-xl border border-white/5 bg-white/5 p-4 space-y-2">
                <div className="text-gray-500">当前聚焦工作项</div>
                {relatedWorkItems.length > 0 ? (
                  relatedWorkItems.slice(0, 2).map((workItem) => (
                    <div key={workItem.id} className="text-gray-300">
                      <span className="text-white font-semibold">{workItem.id}</span> · {workItem.title}
                    </div>
                  ))
                ) : (
                  <div className="text-gray-400">暂无直接关联工作项，当前以会话上下文为主。</div>
                )}
              </div>
            </div>

            <div className="text-gray-500 border-l-2 border-white/5 pl-4 py-1 space-y-1">
              <p>[{formatTimeZh(activeSession.created_at)}] 已接入运行时：{getRuntimeLabel(activeSession.runtime)}</p>
              <p>[{formatTimeZh(activeSession.created_at)}] 已建立项目权威源：{activeProjectId || '未选择项目'} / {currentData?.events.length || 0} 条事件</p>
              <p>[{formatTimeZh(activeSession.created_at)}] 当前席位：{activeSeat?.name || activeSession.seat_id}</p>
            </div>

            <div className="space-y-2">
              <div className="flex items-center gap-2 text-white font-bold">
                <Play size={12} className="fill-current text-primary" />
                今日执行片段
              </div>
              <div className="pl-6 text-gray-300 space-y-2">
                {feed.map((line) => (
                  <p key={line} className="flex items-start gap-2">
                    <ChevronRight size={10} className="mt-1 text-gray-500" />
                    <span>{line}</span>
                  </p>
                ))}
              </div>
            </div>

            {activeSession.status === 'InputRequired' ? (
              <div className="flex gap-2 mt-8 text-status-warning font-bold animate-pulse">
                <span>[代理等待输入]</span>
                <span>当前会话需要你明确边界、优先级或下一步动作后再继续。</span>
                <span className="bg-status-warning w-2 h-4 animate-pulse" />
              </div>
            ) : (
              <div className="flex gap-2 mt-8">
                <span className="bg-primary w-2 h-5 animate-pulse" />
                <span className="text-gray-500 italic">会话保持在线，等待新的输入、交接或文件写回。</span>
              </div>
            )}
          </div>
        ) : (
          <div className="h-full flex flex-col items-center justify-center opacity-25 text-white space-y-4">
            <Terminal size={64} strokeWidth={1} />
            <p className="text-sm font-black tracking-[0.28em]">暂无运行中的会话</p>
            <p className="text-[11px] font-semibold tracking-wide text-gray-500">请先选择席位启动新会话，或接入一个已存在的本地进程。</p>
          </div>
        )}
      </div>

      <div className="px-6 py-2 border-t border-white/5 bg-black/20 flex justify-between text-[10px] font-semibold text-gray-500 tracking-wide">
        <div className="flex gap-4">
          <span>终端行数 42</span>
          <span>终端列数 120</span>
          <span>项目：{activeProjectId || '未选择'}</span>
        </div>
        <div className="flex gap-4">
          <span>Ctrl+C 中止</span>
          <span>Ctrl+L 清屏</span>
          <span>Ctrl+K 切项目</span>
        </div>
      </div>

      <WrapLaunchDialog isOpen={showWrapLaunch} onClose={() => setShowWrapLaunch(false)} />
      <AttachSessionDialog isOpen={showAttach} onClose={() => setShowAttach(false)} />
    </div>
  );
};

export default TerminalPanel;
