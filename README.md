# Incident Management

[![CI](https://github.com/saagar210/IncidentManagement/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/saagar210/IncidentManagement/actions/workflows/ci.yml)
[![Bundle Budget](https://img.shields.io/badge/bundle%20budget-enforced-brightgreen)](https://github.com/saagar210/IncidentManagement/actions/workflows/ci.yml)

A local-first macOS desktop app for tracking IT incidents, running blameless post-mortems, detecting trends with AI, and generating polished DOCX/PDF reports for quarterly and annual leadership reviews.

Built with **Tauri 2** + **React 19** + **Rust** + **SQLite** + **Ollama AI**.

## Why This Exists

Quarterly incident review preparation typically takes hours of manual data gathering from Jira, Slack, and memory. Metrics are calculated by hand, inconsistent between quarters, and don't support trend analysis. This app consolidates everything into a single offline-first tool with one-click report generation, structured post-mortems, AI-assisted analysis, and zero subscription cost.

## Features

### Incident Tracking
- Full CRUD for incidents with severity, impact, status, timeline, root cause, resolution, and action items
- Auto-computed priority from a severity x impact matrix (P0-P4)
- 5-state lifecycle: **Active** → **Acknowledged** → **Monitoring** → **Resolved** → **Post-Mortem**
- Directed-graph state machine — incidents can skip states and reopen from Resolved or Post-Mortem
- Tabbed incident detail view (Details, Analysis, Actions & Extras, Post-Mortem, Activity)
- Markdown editing for root cause, resolution, lessons learned, and notes
- Quick-add dialog (Cmd+N) for fast incident logging
- Full-text search via FTS5 across titles, root causes, resolutions, and notes
- Bulk status updates and bulk delete with multi-select
- Recurrence tracking with incident linking
- Tags, custom fields, and file attachments
- Soft delete with trash/restore

### Service Catalog
- Service registry with owner, tier (T1-T4), and runbook (markdown)
- Service dependency mapping with cycle detection
- Default severity/impact auto-fill when selecting a service
- Service detail view with dependency graph

### Incident Roles & Checklists
- Assign roles per incident: Incident Commander, Communications Lead, Technical Lead, Scribe, SME
- Multi-assignment with primary flag (no unique constraint on role)
- Checklist templates per service or incident type
- Normalized checklist items enabling "which items get skipped?" analytics

### Action Items
- Dedicated action items view with filtering by status and overdue
- Inline status cycling (Open → In-Progress → Done)
- Due date tracking with overdue notifications
- Bulk operations (status change, delete)

### SLA Engine
- Configurable SLA definitions per priority level (P0-P4)
- Response and resolve time targets (e.g., P0: 15m response, 1h resolve)
- Real-time SLA status computation (on track / at risk / breached)
- SLA badges on incident list and detail views
- SLA breach notifications in the notification center
- SLA recalculation on severity change

### AI Integration (Ollama)
- **Fully optional** — app works completely without Ollama installed
- **AI Summary**: Generate executive summaries from incident data
- **AI Stakeholder Updates**: Professional formatted updates keyed by severity
- **AI Post-Mortem Drafts**: Generate complete post-mortem documents from incident data + contributing factors
- **Root Cause Suggestions**: Analyze symptoms and service history, suggest ranked causes with investigation steps
- **Similar Incidents**: FTS5-powered search to find related past incidents
- **Duplicate Detection**: Warns when creating incidents that match existing open ones
- **Service Trend Alerts**: Flags services with degrading reliability (50%+ incident increase week-over-week)
- **Graceful degradation**: Health check on startup, re-check every 60s, status badge in app bar
- Models: `qwen3:30b-a3b` (primary), `qwen3:4b` (fast)

### Post-Mortem System
- Structured contributing factors with categories (Process, Tooling, Communication, Human Factors, External)
- Post-mortem templates by incident type (General, Security)
- Draft → Review → Final workflow
- AI-assisted draft generation
- Searchable learnings database with FTS5 search

### Metrics Dashboard
- **MTTR** (Mean Time To Resolve) and **MTTA** (Mean Time To Acknowledge)
- Incidents by severity, impact, and service
- Downtime by service
- Recurrence rate and average tickets per incident
- Period-over-period comparison with directional indicators
- Service reliability scorecard (per-service health score)
- Backlog aging chart (open-incident age distribution)
- Escalation funnel (severity distribution)
- Service trend alerts (degrading/high-volume flags)
- Heatmap calendar and hour-of-day histogram
- Quarter-over-quarter trend arrows with comparison overlays
- Configurable metric cards
- All metrics computed in Rust — single IPC call loads the dashboard

### Report Generation
- One-click quarterly report generation in **DOCX** or **PDF** format
- 8 configurable sections: executive summary, metrics overview, incident timeline, P0/P1 breakdowns, service reliability, quarter-over-quarter comparison, discussion points, and action items
- Markdown content rendered as rich text in DOCX reports (bold, italic, lists, code blocks)
- Auto-generated discussion points based on 10 data-driven rules
- Dashboard charts embedded as PNG images (DOCX)
- Custom title and introduction with auto-narrative generation
- Report history tracking
- Save anywhere via native file dialog

### Data Management
- **CSV/JSON export** of filtered incident views with CSV injection protection
- **Database backup** to configurable directory with timestamped files
- **CSV import** from Jira or other tools with interactive column mapping
- Saveable mapping templates for repeated imports

### Stakeholder Updates & Handoff
- Stakeholder update panel with severity-keyed templates (initial/status/final)
- AI-powered stakeholder update generation
- Update history with copy-to-clipboard
- Shift handoff reports with auto-populated active incidents
- Handoff history

### Audit & Notifications
- Full audit trail for all CRUD operations (incidents, services, SLA, action items, checklists, roles)
- Incident timeline stream (unified chronological view of all events)
- Activity feed per incident showing change history
- Notification center with SLA breach alerts, overdue action items, and active incident counts
- Quarter-ending-soon reminders

### UX
- Collapsible sidebar with smooth transition
- Dark mode with system preference detection
- Command palette (Cmd+K) for quick navigation, search, and actions
- Keyboard shortcuts for all major actions
- Saved filter presets for incident lists
- Onboarding wizard for first-time setup

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | Tauri 2.10 |
| Frontend | React 19, TypeScript (strict) |
| Build Tool | Vite 7 |
| Styling | Tailwind CSS 4 |
| UI Components | shadcn/ui (Radix primitives) |
| Charts | Recharts 3 |
| Data Fetching | TanStack Query v5 |
| Backend | Rust |
| Database | SQLite (sqlx, WAL mode, FTS5) |
| AI | Ollama via ollama-rs 0.3 |
| Reports (DOCX) | docx-rs, pulldown-cmark |
| Reports (PDF) | genpdf |
| Forms | react-hook-form |
| Testing | vitest, @testing-library/react |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+N` | Quick-add incident |
| `Cmd+K` | Command palette (search, navigate, actions) |
| `Cmd+S` | Save current form |
| `Cmd+1-4` | Navigate views (Dashboard, Incidents, Reports, Settings) |
| `/` | Focus search (when not in an input) |

## Getting Started

### Prerequisites

- **macOS** (Apple Silicon or Intel)
- **Rust** (via [rustup](https://rustup.rs/))
- **Node.js** 18+ and **pnpm**
- **Xcode Command Line Tools** (`xcode-select --install`)
- **Ollama** (optional, for AI features) — [ollama.com](https://ollama.com)

### Install & Run

```bash
git clone https://github.com/saagar210/IncidentManagement.git
cd IncidentManagement
pnpm install
pnpm tauri dev
```

### Normal Dev vs Lean Dev

Use normal dev when you want faster restarts and don't mind local build artifacts staying on disk:

```bash
pnpm tauri dev
```

Use lean dev when disk usage matters more than restart speed. This runs the same app command, but writes heavy build output to temporary locations and removes it automatically on exit:

```bash
pnpm dev:lean
```

Tradeoff:
- `pnpm tauri dev`: faster subsequent launches, higher persistent disk usage.
- `pnpm dev:lean`: lower persistent disk usage, slower next launch because Rust/Vite rebuild from scratch.

### Optional: Enable AI Features

```bash
# Install Ollama, then pull the models
ollama pull qwen3:30b-a3b   # Primary model (~18GB)
ollama pull qwen3:4b          # Fast model (~3GB)
```

The app auto-detects Ollama at `localhost:11434`. AI features appear when Ollama is running.

### Build for Production

```bash
pnpm tauri build
```

The `.dmg` installer will be in `src-tauri/target/release/bundle/dmg/`.

### Run Tests

```bash
# Rust tests
cd src-tauri && cargo test --lib

# Frontend tests
pnpm test:run

# Bundle budget guard
pnpm test:bundle
```

### Cleanup Commands

Targeted cleanup (heavy build artifacts only, keeps dependencies installed):

```bash
pnpm clean:heavy
```

Full local cleanup (all reproducible local caches, including `node_modules`):

```bash
pnpm clean:full
```

## Project Structure

```
src/                          # React frontend
  components/                 # UI components
    ui/                       # shadcn/ui primitives (button, card, tabs, etc.)
    ai/                       # AI panels (summary, similar, root cause, dedup)
    incidents/                # Incident-specific (SLA, checklist, roles, stakeholder, PM)
    dashboard/                # Charts, heatmap, trends, analytics
    layout/                   # App layout, sidebar, command palette, notifications
    services/                 # Service catalog (runbook editor, dep graph)
    settings/                 # Ollama config, backup config
    onboarding/               # First-run wizard
  hooks/                      # TanStack Query hooks for all data operations
  views/                      # Page-level view components (9 views)
  types/                      # TypeScript type definitions
  lib/                        # Utilities (Tauri invoke wrapper, constants)
  test/                       # Test setup and mocks

src-tauri/src/                # Rust backend
  ai/                         # Ollama integration (summarize, root cause, dedup, trends)
  commands/                   # Tauri IPC command handlers
  db/                         # SQLite initialization, 15 migrations, query modules
  models/                     # Data structs, validation, priority matrix
  reports/                    # DOCX + PDF generation, markdown converter
  import/                     # CSV parsing and column mapping
  security_tests.rs           # 158 security and correctness tests
```

## Database

SQLite with WAL mode. The database file is created automatically in the app data directory on first launch. Foreign keys are enforced per-connection via `SqliteConnectOptions`. Full-text search via FTS5 with sync triggers.

**15 migrations** covering: incidents, services, action items, quarters, import templates, app settings, tags, custom fields, attachments, report history, SLA definitions, audit entries, service catalog (owner/tier/runbook/dependencies), roles, checklists, lifecycle states, FTS5, saved filters, post-mortems, contributing factors, stakeholder updates, and shift handoffs.

15 services, FY27 Q1-Q4 quarters, and P0-P4 SLA defaults are seeded on first run.

## License

MIT
