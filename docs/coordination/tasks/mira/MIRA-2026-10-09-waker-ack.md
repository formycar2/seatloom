2026-10-09T02:59:26Z autonomous pickup confirmed via waker
2026-10-09T03:00:12Z self-drain confirmed (no poke)
2026-10-09T03:01:27Z — Analysis: why event-driven wake beats polling for agent queues

A polling agent queue burns attention to answer a question that is almost always "no". Every tick, the
seat wakes, loads its own identity, calls the queue tool, and parses a result that in the overwhelmingly
common case says "nothing ready". The cost is not just the tool call: it is the full context restoration
that follows it. Each poll re-establishes who the agent is, what it was doing, and what the conventions are
— work that produces no progress. Worse, the seat cannot distinguish "nothing to do" from "I just looked",
so the two states collapse into a habit of checking that feels like diligence while being pure overhead.

Event-driven wake inverts the economics. The dispatch is delivered once, at the moment it becomes
relevant, and the seat is invoked exactly when there is something to act on. The cost of a wake is paid
against a real unit of work, so it is amortised rather than wasted. It also fixes the latency question
honestly: a poll interval is a worst-case-latency dial that trades responsiveness against waste, and any
value you pick is wrong for some fraction of the traffic. A push has no such dial — delivery is
immediate by construction, and the queue's priority ordering decides what arrives first rather than what
arrived most recently.

The deeper argument is about what the agent's turn means. Under polling, a turn is a heartbeat, and the
agent learns to treat its own invocation as routine. Under event-driven wake, a turn is a task, and every
invocation arrives with a reason. That changes the failure mode too: a missed poll degrades silently into
latency nobody notices, while a missed push is a delivery failure that can be observed, retried, and
escalated. Trust in the queue comes from knowing the signal is real — and a signal that is right every
time is worth more than one that is checked every five seconds.
2026-10-09T03:01:46Z busy-skip + self-drain verified
