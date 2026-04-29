import React, { useState, useEffect, useRef } from 'react';
import {
  Search,
  Plus,
  HelpCircle,
  X,
  CheckCircle2,
  AlertTriangle,
  User,
  Link2,
  Coins,
  Zap,
  ArrowRight,
  Edit3,
  Clock,
} from 'lucide-react';
import { useDataStore } from '../stores/useDataStore';

interface SupervisorCommandBarProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirmProposal: (workItem: any) => void;
}

type CommandMode = 'create' | 'explain' | 'evidence';

const RECENT_COMMANDS = [
  'Create WorkItem for UI contract repair',
  'Explain why the terminal is blocked in ses-407',
  'Find evidence for US-P0-04 delegation',
  'Route new fix to Nimbus',
];

const SupervisorCommandBar: React.FC<SupervisorCommandBarProps> = ({
  isOpen,
  onClose,
  onConfirmProposal,
}) => {
  const [input, setInput] = useState('');
  const [mode, setCommandMode] = useState<CommandMode>('create');
  const [isProcessing, setIsProcessing] = useState(false);
  const [proposal, setProposal] = useState<any | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [isFocused, setIsFocused] = useState(false);
  
  const inputRef = useRef<HTMLInputElement>(null);

  // Global keyboard listener for Enter/Escape when modal is open
  useEffect(() => {
    if (!isOpen) return;

    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Escape always closes
      if (e.key === 'Escape') {
        onClose();
        return;
      }

      // Handle Enter logic
      if (e.key === 'Enter') {
        // 1. If in inline edit mode, let the local handler deal with it (bubbling usually fine)
        if (isEditing) return;

        // 2. If a valid proposal is open, confirm it (Solves the disabled input blocker)
        if (proposal && !isProcessing && !proposal.placeholder) {
          e.preventDefault();
          handleConfirm();
          return;
        }

        // 3. If no proposal and input is present, generate proposal
        if (!proposal && !isProcessing && input.trim()) {
          e.preventDefault();
          generateProposal();
          return;
        }
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, [isOpen, input, proposal, isProcessing, isEditing]); // Re-bind when state changes

  useEffect(() => {
    if (isOpen) {
      setInput('');
      setProposal(null);
      setIsProcessing(false);
      setIsEditing(false);
      setIsFocused(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const generateProposal = () => {
    setIsProcessing(true);
    // Simulate deterministic local parsing/synthesis
    setTimeout(() => {
      setIsProcessing(false);
      if (mode === 'create') {
        setProposal({
          intent: input,
          title: input.length > 30 ? input.substring(0, 30) + '...' : input,
          owner: 'seat-2', // Defaulting to Nimbus for prototype
          ownerName: 'nimbus',
          rationale: 'Nimbus has the Architect role and matching capabilities for engineering design and baseline alignment.',
          evidence: ['docs/prd-v0.5.md', 'docs/architecture-design.md'],
          ac: [
            'Drafted implementation must follow v0.5 protocol.',
            'Linked evidence must be valid and present.',
            'State changes must be recorded in the coordination ledger.'
          ],
          budget: '150,000 tokens',
          impact: 'Will create a new WorkItem assigned to nimbus and record an audit event in the timeline.'
        });
      } else {
        // Explain and Evidence modes show placeholder cards
        setProposal({ placeholder: true });
      }
    }, 800);
  };

  const handleConfirm = () => {
    if (!proposal || proposal.placeholder) return;
    
    // Create a local WorkItem draft
    const newWI = {
      id: `wi-${Math.floor(Math.random() * 900 + 400)}`,
      title: proposal.title,
      goal: proposal.intent,
      acceptance_criteria: proposal.ac,
      owner_seat_id: proposal.owner,
      status: 'Draft',
      priority: 'Normal',
      depends_on: [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    
    onConfirmProposal(newWI);
    onClose();
  };

  const selectRecent = (cmd: string) => {
    setInput(cmd);
    setIsFocused(false);
    // Auto-generate if it feels like a full intent
    setTimeout(() => generateProposal(), 100);
  };

  return (
    <div className="fixed inset-0 z-[200] flex items-start justify-center pt-[15vh] px-4">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-secondary/40 backdrop-blur-sm" onClick={onClose} />
      
      {/* Command Bar Container */}
      <div className="relative w-full max-w-[640px] flex flex-col gap-2 animate-in slide-in-from-top-4 duration-300">
        <div className="bg-card border border-border shadow-2xl rounded-2xl overflow-hidden flex flex-col">
          {/* Input Area */}
          <div className="flex items-center px-4 h-14 gap-3 border-b border-border/60">
            <Search className={`w-5 h-5 ${isProcessing ? 'text-primary animate-pulse' : 'text-ink-faint'}`} />
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onFocus={() => setIsFocused(true)}
              disabled={isProcessing || proposal}
              placeholder="Ask SeatLoom to create, route, review, recover, or explain work..."
              className="flex-1 bg-transparent border-none outline-none text-sm font-bold text-ink placeholder:text-ink-faint"
            />
            {input && !isProcessing && !proposal && (
              <button onClick={() => setInput('')} className="text-ink-faint hover:text-ink">
                <X size={16} />
              </button>
            )}
            {proposal && (
              <button onClick={() => { setProposal(null); setInput(''); setTimeout(() => inputRef.current?.focus(), 50); }} className="text-ink-faint hover:text-ink">
                <X size={16} />
              </button>
            )}
          </div>

          {/* Recent Commands Dropdown */}
          {isFocused && !input && !proposal && !isProcessing && (
            <div className="p-2 border-b border-border/60 bg-secondary/10">
              <div className="px-3 py-1 mb-1">
                <span className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Recent Intents</span>
              </div>
              <div className="flex flex-col">
                {RECENT_COMMANDS.map((cmd, idx) => (
                  <button
                    key={idx}
                    onClick={() => selectRecent(cmd)}
                    className="flex items-center gap-3 px-3 py-2 rounded-xl hover:bg-primary/5 text-left transition-all group"
                  >
                    <Clock size={12} className="text-ink-faint group-hover:text-primary" />
                    <span className="text-[11px] font-bold text-ink-soft group-hover:text-ink">{cmd}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Mode Selector */}
          {!proposal && !isProcessing && (
            <div className="flex items-center gap-1 p-2 bg-secondary/30">
              <button 
                onClick={() => setCommandMode('create')}
                className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all flex items-center gap-1.5 ${
                  mode === 'create' ? 'bg-primary text-surface shadow-sm' : 'hover:bg-accent text-ink-soft'
                }`}
              >
                <Plus size={12} />
                Create
              </button>
              <button 
                onClick={() => setCommandMode('explain')}
                className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all flex items-center gap-1.5 ${
                  mode === 'explain' ? 'bg-primary text-surface shadow-sm' : 'hover:bg-accent text-ink-soft'
                }`}
              >
                <HelpCircle size={12} />
                Explain
              </button>
              <button 
                onClick={() => setCommandMode('evidence')}
                className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all flex items-center gap-1.5 ${
                  mode === 'evidence' ? 'bg-primary text-surface shadow-sm' : 'hover:bg-accent text-ink-soft'
                }`}
              >
                <Link2 size={12} />
                Find Evidence
              </button>
            </div>
          )}
        </div>

        {/* Suggestion Card */}
        {proposal && !proposal.placeholder && (
          <div className="bg-card border border-border shadow-2xl rounded-2xl overflow-hidden animate-in zoom-in-95 duration-300 mt-2">
            <div className="bg-primary/5 px-6 py-4 border-b border-border/60 flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Zap size={16} className="text-primary" />
                <span className="text-[10px] font-black text-ink uppercase tracking-widest">Supervisor Proposal</span>
              </div>
              <span className="text-[10px] font-bold text-ink-faint">V0.5 PROTOCOL</span>
            </div>

            <div className="p-6 space-y-5">
              <div className="space-y-1">
                <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Intent</div>
                <div className="text-sm font-bold text-ink">"{proposal.intent}"</div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-3">
                  <div className="space-y-1">
                    <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Proposed WorkItem</div>
                    <div className="text-xs font-black text-primary bg-accent px-2 py-1 rounded border border-primary/10 inline-block w-full">
                      {isEditing ? (
                        <input 
                          type="text" 
                          autoFocus
                          value={proposal.title} 
                          onChange={(e) => setProposal({...proposal, title: e.target.value})}
                          onKeyDown={(e) => e.key === 'Enter' && setIsEditing(false)}
                          className="bg-transparent border-none outline-none w-full font-black text-primary"
                        />
                      ) : proposal.title}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Ownership</div>
                    <div className="flex items-center gap-2">
                      <div className="w-6 h-6 rounded-full bg-secondary flex items-center justify-center border border-border">
                        <User size={12} className="text-ink-soft" />
                      </div>
                      <span className="text-[11px] font-bold text-ink">{proposal.ownerName}</span>
                      <span className="text-[10px] text-ink-soft opacity-60">· Architect</span>
                    </div>
                    <p className="text-[10px] text-ink-soft leading-relaxed italic mt-1">{proposal.rationale}</p>
                  </div>
                </div>

                <div className="space-y-3">
                  <div className="space-y-1">
                    <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Linked Evidence</div>
                    <div className="flex flex-wrap gap-1.5">
                      {proposal.evidence.map((path: string) => (
                        <div key={path} className="flex items-center gap-1 text-[10px] font-bold text-ink-soft bg-secondary px-1.5 py-0.5 rounded border border-border/40">
                          <Link2 size={10} />
                          {path.split('/').pop()}
                        </div>
                      ))}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest flex items-center gap-1">
                      <Coins size={10} />
                      Budget Estimate
                    </div>
                    <div className="text-[11px] font-black text-primary">{proposal.budget}</div>
                  </div>
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest">Acceptance Criteria</div>
                <ul className="space-y-1.5">
                  {proposal.ac.map((item: string, idx: number) => (
                    <li key={idx} className="flex items-start gap-2 text-[11px] font-bold text-ink-soft">
                      <div className="mt-1.5 w-1 h-1 rounded-full bg-primary shrink-0" />
                      {item}
                    </li>
                  ))}
                </ul>
              </div>

              <div className="bg-secondary/40 p-3 rounded-xl border border-border/40">
                <div className="text-[9px] font-black text-ink-faint uppercase tracking-widest mb-1">Impact Summary</div>
                <p className="text-[11px] font-bold text-ink-soft leading-relaxed">{proposal.impact}</p>
              </div>
            </div>

            <div className="bg-secondary/30 px-6 py-4 border-t border-border/60 flex justify-between items-center">
              <button 
                onClick={() => { setProposal(null); inputRef.current?.focus(); }}
                className="text-[10px] font-black text-ink-soft uppercase tracking-widest hover:text-ink"
              >
                Cancel
              </button>
              <div className="flex gap-2">
                <button 
                  onClick={() => setIsEditing(!isEditing)}
                  className={`px-4 py-2 rounded-xl text-[10px] font-black uppercase tracking-widest transition-all flex items-center gap-1.5 border border-border/60 ${
                    isEditing ? 'bg-accent text-primary' : 'bg-card text-ink-soft hover:bg-accent'
                  }`}
                >
                  <Edit3 size={12} />
                  {isEditing ? 'Finish Edit' : 'Edit Inline'}
                </button>
                <button 
                  onClick={handleConfirm}
                  className="px-6 py-2 bg-primary text-surface rounded-xl text-[10px] font-black uppercase tracking-widest flex items-center gap-2 shadow-lg shadow-primary/20 hover:opacity-90 transition-all"
                  title="Press Enter to confirm"
                >
                  <CheckCircle2 size={14} />
                  Confirm Proposal (Enter)
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Placeholder for Explain/Evidence */}
        {proposal && proposal.placeholder && (
          <div className="bg-card border border-border shadow-2xl rounded-2xl p-8 flex flex-col items-center text-center space-y-4 animate-in zoom-in-95 duration-300 mt-2">
            <div className="p-4 bg-primary/10 text-primary rounded-full">
              {mode === 'explain' ? <HelpCircle size={32} /> : <Link2 size={32} />}
            </div>
            <div className="space-y-2">
              <h3 className="text-lg font-black text-ink uppercase">{mode === 'explain' ? 'Explain Mode' : 'Evidence Search'}</h3>
              <p className="text-sm text-ink-soft font-bold max-w-[300px]">
                {mode === 'explain' 
                  ? "Decision grounding logic is currently blocked pending the Data Engine recall module." 
                  : "Retrieval indexing is in progress. Exact match filtering is active in the Timeline and Inbox views."}
              </p>
            </div>
            <button 
              onClick={() => { setProposal(null); inputRef.current?.focus(); }}
              className="px-6 py-2 bg-secondary text-ink font-black text-[10px] uppercase tracking-widest rounded-xl hover:bg-accent transition-all"
            >
              Back to Command Bar
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

export default SupervisorCommandBar;
