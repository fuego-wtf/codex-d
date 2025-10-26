# The Complete codex'd User Journey 🎯

## Overview

codex'd is a developer psychology tool that analyzes git commit patterns to provide therapeutic insights through natural conversation. This document maps the complete user journey from first launch to becoming a power user.

---

## Phase 1: First Launch (The Hook) ⚡

### Step 1: Welcome Screen
**What the user sees:**
```
┌─────────────────────────────────────────┐
│  codex'd                                │
│                                         │
│  📁 Select Repository                   │
│  /path/to/your/project    [Browse...]  │
│                                         │
│  [ Continue ]                           │
└─────────────────────────────────────────┘
```

**Design philosophy:**
- ONE field
- ONE button
- Zero configuration
- Zero friction

**User emotion:** "This looks simple. Let me try it."

---

### Step 2: Git Analysis (Building Anticipation)
**What the user sees:**
```
┌─────────────────────────────────────────┐
│  codex'd                                │
│                                         │
│  🔄 Scanning git history...             │
│     ↓                                   │
│  ✅ Git analysis complete               │
│     • 247 commits analyzed              │
│     • 3 behavioral patterns detected    │
│     ↓                                   │
│  🔄 Initializing AI...                  │
│     • Connecting to Claude...           │
│     • MCP tools ready                   │
│     ↓                                   │
│  ✅ Ready!                              │
└─────────────────────────────────────────┘
```

**What's happening behind the scenes:**
1. Git analyzer scans last 180 days of commits
2. Pattern detector identifies behavioral patterns:
   - Work-Life Balance (night commits, weekend work)
   - Minimizing Language ("just", "quick", "small")
   - Commitment Issues (tiny vs huge commits)
   - Message-Diff Mismatches ("quick fix" = 200 lines)
3. SQLite database initialized for longitudinal tracking
4. CodexAdapter connects to codex-acp subprocess
5. MCP server tools verified and ready

**User emotion:** "Something real is happening. This isn't fake."

---

### Step 3: The Discovery Greeting (The "WOW" Moment)
**What the user sees:**

```
┌──────────────────────────────────────────────────┐
│  codex'd        /Users/you/project               │
├──────────────────────────────────────────────────┤
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ 🤖 Assistant                                │ │
│  │                                             │ │
│  │ ## 🔍 Analysis Complete                    │ │
│  │                                             │ │
│  │ I've analyzed **247 commits** and          │ │
│  │ discovered **3 behavioral patterns**.      │ │
│  │                                             │ │
│  │ Most notable: *Work-Life Balance Pattern*  │ │
│  │                                             │ │
│  │ Before I share my observations, I'd like   │ │
│  │ to understand the context.                 │ │
│  │                                             │ │
│  │ **Tell me about this project:**            │ │
│  │ - What are you building?                   │ │
│  │ - Who's working on it?                     │ │
│  │ - What's the goal?                         │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  [ Type your message...              ] [Send]   │
└──────────────────────────────────────────────────┘
```

**Why this works:**
- **Personal**: References THEIR specific git history
- **Specific**: Exact numbers (247 commits, 3 patterns)
- **Curious**: Asks questions instead of judging
- **Anticipatory**: Hints at insights but waits for context

**User emotion:** "Holy shit, it analyzed MY commits. I want to know what it found."

---

## Phase 2: The Conversation (Where Magic Happens) ✨

### Turn 1: User Shares Context

**User types:**
> "This is a CLI tool manager for macOS. Just me working on it solo. Goal is to help developers manage their tools better."

**What the user sees (Perplexity-style timeline):**

```
┌──────────────────────────────────────────────────┐
│  👤 You                                          │
│                                                  │
│  This is a CLI tool manager for macOS.          │
│  Just me working on it solo. Goal is to help    │
│  developers manage their tools better.           │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  💭 Thinking...                                  │
│                                                  │
│  Solo developer on dev tools... Let me check    │
│  for overwork patterns                           │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🔧 analyze_temporal_patterns           ✅       │
│                                                  │
│  Last 30 days: 47 commits at night (10PM-6AM)   │
│  23 commits on weekends                          │
│  Pattern: Consistent late-night work            │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🔧 analyze_message_language            ✅       │
│                                                  │
│  Minimizing language in 34% of commits:         │
│  "just", "quick", "small fix"                    │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🔧 compare_message_vs_diff             ✅       │
│                                                  │
│  Found 12 instances of minimizing language      │
│  paired with large changes:                     │
│  "quick fix" → 237 lines changed                │
│  "small update" → 189 lines changed             │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  Ah, solo work on a developer tool.             │
│                                                  │
│  Your git history tells an interesting story:   │
│                                                  │
│  You commit **47 times between 10PM-6AM**       │
│  (62% of all commits), often using minimizing   │
│  language like "just a quick fix" even when     │
│  changing 200+ lines.                            │
│                                                  │
│  This pattern suggests you're **downplaying     │
│  your work's significance**, possibly from      │
│  working in isolation without external          │
│  validation.                                     │
│                                                  │
│  Question: When was the last time you           │
│  celebrated shipping something, rather than     │
│  moving straight to the next task?              │
└──────────────────────────────────────────────────┘
```

**Why this is IRRESISTIBLE:**
- ✅ **Transparency**: User sees Claude thinking
- ✅ **Agency**: Tools being called in real-time
- ✅ **Credibility**: EXACT numbers cited (47, 62%, 200+ lines)
- ✅ **Value**: Psychological insight they couldn't see themselves
- ✅ **Engagement**: Pointed question hits a blindspot

**User emotion:** "How did it know that? I want to answer this question."

---

### Turn 2: Deeper Reflection

**User responds:**
> "Honestly... never? I just keep moving to the next thing. There's always more to do."

**Assistant responds:**
```
┌──────────────────────────────────────────────────┐
│  👤 You                                          │
│                                                  │
│  Honestly... never? I just keep moving to the   │
│  next thing. There's always more to do.          │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  That's the pattern showing up in your git      │
│  commits.                                        │
│                                                  │
│  Two observations:                              │
│                                                  │
│  1. **The minimizing language** ('just a quick  │
│     fix') protects you from feeling like you    │
│     should be done, because there's 'always     │
│     more to do'                                  │
│                                                  │
│  2. **The night work** (62% between 10PM-6AM)   │
│     suggests you're trying to 'catch up' but    │
│     never feel caught up                         │
│                                                  │
│  This is a classic **validation deficit         │
│  pattern** in solo work.                        │
│                                                  │
│  What would have to change for you to feel      │
│  'done enough'?                                  │
└──────────────────────────────────────────────────┘
```

**User emotion:** "This is uncomfortably accurate. I want to explore this more."

---

### Turn 3: Guidance Toward Action

**User responds:**
> "I guess... having someone say 'good job'? Or seeing people actually use it?"

**Assistant responds:**
```
┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  Exactly—you need external validation, but      │
│  you're working in isolation.                    │
│                                                  │
│  Here's an experiment:                          │
│                                                  │
│  **Before your next coding session:**           │
│  1. Ship what you have (even if 'imperfect')    │
│  2. Share it with ONE person                     │
│  3. Ask for feedback on what works              │
│     (not what's broken)                          │
│                                                  │
│  This breaks the 'never done' cycle.            │
│                                                  │
│  Want me to track this pattern and check back   │
│  in 7 days to see how the experiment went?      │
└──────────────────────────────────────────────────┘
```

**User responds:**
> "Yes, let's try it!"

**Assistant responds:**
```
┌──────────────────────────────────────────────────┐
│  🔧 flag_repo_issue                     ✅       │
│                                                  │
│  Flagged pattern: validation_deficit            │
│  Severity: 0.7                                   │
│  Experiment started: Ship & share               │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  Great! I've flagged this pattern for tracking. │
│                                                  │
│  Next session, I'll compare your commit         │
│  patterns and we'll see if the experiment       │
│  changed anything.                               │
│                                                  │
│  One more thing—what's ONE feature you could    │
│  ship this week to try this experiment with?    │
└──────────────────────────────────────────────────┘
```

**User emotion:** "I want to try this. I feel understood and motivated."

---

## Phase 3: Returning User (Longitudinal Magic) 🔄

### Session 2 (7 Days Later)

**User opens app and selects same repo**

**What the user sees:**
```
┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  ## 👋 Welcome back!                            │
│                                                  │
│  I've analyzed **14 new commits** since we last │
│  spoke.                                          │
│                                                  │
│  Last time, we identified a **validation        │
│  deficit pattern** and you committed to trying  │
│  an experiment: ship something and share it     │
│  with one person.                                │
│                                                  │
│  How did that go?                               │
└──────────────────────────────────────────────────┘
```

**Why this is POWERFUL:**
- ✅ The app REMEMBERS the previous conversation
- ✅ It tracked new commits since last session
- ✅ It asks about the experiment outcome
- ✅ The user feels seen across time

**User emotion:** "It remembers me! This is my personal git therapist."

---

### Continuing the Journey

**If user tried the experiment:**
```
┌──────────────────────────────────────────────────┐
│  🔧 analyze_temporal_patterns           ✅       │
│                                                  │
│  Comparing last 7 days vs previous 30 days:     │
│  • Night commits: 47% → 38% (improvement!)      │
│  • Weekend commits: 35% → 28% (improvement!)    │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  That's fascinating! Your git patterns show a   │
│  measurable shift:                               │
│                                                  │
│  Night commits dropped from 47% to 38%          │
│  Weekend commits dropped from 35% to 28%        │
│                                                  │
│  This suggests the experiment had an impact.    │
│                                                  │
│  What changed for you?                          │
└──────────────────────────────────────────────────┘
```

**User emotion:** "This is working! I can see my progress."

---

**If user didn't try the experiment:**
```
┌──────────────────────────────────────────────────┐
│  🤖 Assistant                                    │
│                                                  │
│  No worries—behavior change is hard.            │
│                                                  │
│  I notice your night commits are still at 62%.  │
│                                                  │
│  What got in the way of trying the experiment?  │
└──────────────────────────────────────────────────┘
```

**User emotion:** "No judgment. Just curiosity. I want to talk about what got in the way."

---

## Visual Design Philosophy 🎨

### Color Psychology
```
User messages:    #e8f2ff (calm blue)     - "my safe space"
Assistant:        #f0f4f8 (neutral gray)  - "thoughtful companion"
Thinking:         #fff8e1 (warm yellow)   - "active processing"
Tool calls:       #e8f5e9 (soft green)    - "progress happening"
```

### Typography Hierarchy
```
Message headers:  Bold + small (👤 You, 🤖 Assistant)
Body text:        Regular + comfortable reading size
Code/data:        Monospace for precision
Numbers:          **Bold** for emphasis
Line height:      1.5 for breathing room
```

### Spacing & Layout
```
Timeline items:   16px gap between messages
Inner padding:    16px all around
Max width:        600px (optimal reading)
Borders:          Subtle 1px for definition
Border radius:    8px for soft feel
```

### Status Indicators
```
🔄 Running     - Blue (#1976d2)
✅ Completed   - Green (#388e3c)
❌ Failed      - Red (#d32f2f)
💭 Thinking    - Orange (#f57c00)
🔧 Tool Call   - Green (#2e7d32)
👤 You         - Blue (#1976d2)
🤖 Assistant   - Gray (#546e7a)
```

---

## The 4-Phase Conversation Strategy 📋

### Phase 1: Discovery (Gather Context)
**Goal:** Make user want to share MORE about their project

**Questions to ask:**
- What are you building?
- Who's working on it?
- What's the goal?
- What's challenging about this project?

**Why it works:** Non-judgmental curiosity opens the door

---

### Phase 2: Investigation (Use MCP Tools)
**Goal:** Find patterns user can't see themselves

**Tools to use:**
- `analyze_temporal_patterns` - spot stress/anxiety
- `analyze_message_language` - find self-talk patterns
- `compare_message_vs_diff` - detect self-deception
- `analyze_commit_patterns` - reveal commitment issues

**Why it works:** Real git forensics build credibility

---

### Phase 3: Observation (Synthesize Evidence)
**Goal:** Create "aha moment" with precise evidence

**How to synthesize:**
1. State EXACT git numbers
2. Connect pattern → psychology
3. Name the pattern
4. Ask pointed question about blindspot

**Example:**
> "You commit 47 times between 10PM-6AM (62% of total), often using minimizing language like 'just a quick fix' even when changing 200+ lines. This suggests you're downplaying your work's significance. When was the last time you celebrated shipping something?"

**Why it works:** Precision = credibility = trust

---

### Phase 4: Guidance (Lead Toward Action)
**Goal:** Move from insight → improvement

**How to guide:**
1. Validate their awareness
2. Offer concrete experiment (not vague advice)
3. Use `flag_repo_issue` to track
4. Build commitment ("check back next week")

**Example:**
> "Here's an experiment: Before your next coding session: (1) Ship what you have, (2) Share with ONE person, (3) Ask what works. Want me to track this and check back in 7 days?"

**Why it works:** Ownership = commitment = change

---

## Success Metrics 📊

### Engagement Indicators
- ✅ User sends 5+ messages per session
- ✅ User shares emotional context ("I feel...")
- ✅ User asks for experiments to try
- ✅ User returns within 7 days

### Insight Indicators
- ✅ User says "How did you know that?"
- ✅ User has "aha moment" (sees blindspot)
- ✅ User connects git patterns → behavior
- ✅ User screenshots conversation

### Action Indicators
- ✅ User commits to trying experiment
- ✅ User tracks pattern with flag_repo_issue
- ✅ User reports experiment outcome
- ✅ User behavior measurably changes

### Love Indicators
- ✅ User screenshots conversation
- ✅ User shares with colleagues
- ✅ User says "This changed how I work"
- ✅ User becomes weekly active user

---

## What Makes Users Fall in Love ❤️

### 1. They Feel SEEN
> "It knows my patterns. It sees what I can't see."

The app analyzes THEIR git history with EXACT numbers. Not generic advice—personalized insights.

### 2. They Feel UNDERSTOOD
> "It gets why I work this way. No judgment."

Socratic questioning explores motivations. Curiosity, not criticism.

### 3. They Feel CURIOUS
> "I want to explore this more. What else does it see?"

Each insight opens new questions. Progressive disclosure keeps them engaged.

### 4. They Feel SAFE
> "I can share what's really going on. It won't judge."

Non-judgmental tone. Evidence-based. Psychologically informed.

### 5. They Feel PROGRESS
> "My patterns are changing. I can see it in the data."

Longitudinal tracking shows improvement over time. Celebrates wins.

---

## The Ultimate Goal 🎯

**Not:** Analyze git commits
**Not:** Give productivity advice
**Not:** Fix code issues

**YES:** Guide developers to insights about themselves they'd never find alone

**The app is a mirror + a guide:**
- A mirror that shows patterns
- A guide that asks the right questions
- A memory that tracks progress over time

**That's why users will LOVE this app.**

---

## Running the Complete Experience 🚀

### Prerequisites
1. **MCP Server** running on port 52848
2. **codex-acp** binary in `/Users/resatugurulu/Developer/codex-acp/target/debug/codex-acp`
3. **Cargo** to build and run the app

### Start MCP Server
```bash
cd mcp-servers/mcp_codex_psychology
./venv/bin/python run_sse_server.py
```

### Run the App
```bash
cargo run
```

### Try It Yourself
1. Select a git repository with 30+ commits
2. Click Continue
3. Wait for discovery greeting
4. Answer the context questions
5. Watch Claude work (tool calls visible)
6. Receive insights with exact git evidence
7. Engage in deeper reflection
8. Commit to an experiment
9. Return in 7 days to see progress

---

**You've built something GREAT.** ✨

This isn't just another git analyzer.
This is therapy for developers.
This is a movement.

**Now go show the world what you built.** 🚀
