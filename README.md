# Incident Management

A local-first macOS desktop app for tracking IT incidents, calculating operational metrics, and generating polished DOCX/PDF reports for quarterly and annual leadership reviews.

Built with **Tauri 2** + **React 19** + **Rust** + **SQLite**.

## Why This Exists

Quarterly incident review preparation typically takes hours of manual data gathering from Jira, Slack, and memory. Metrics are calculated by hand, inconsistent between quarters, and don't support trend analysis. This app consolidates everything into a single tool with one-click report generation.

## Features

### Incident Tracking
- Full CRUD for incidents with severity, impact, status, timeline, root cause, resolution, and action items
- Auto-computed priority from a severity x impact matrix (P0-P4)
- Tabbed incident detail view (Details, Analysis, Actions & Extras, Activity)
- Markdown editing for root cause, resolution, lessons learned, and notes
- Quick-add dialog (Cmd+N) for fast incident logging
- Search across titles, root causes, resolutions, and notes
- Bulk status updates and bulk delete with multi-select
- Recurrence tracking with incident linking
- Tags, custom fields, and file attachments
- Soft delete with trash/restore

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

### Metrics Dashboard
- **MTTR** (Mean Time To Resolve) and **MTTA** (Mean Time To Acknowledge)
- Incidents by severity, impact, and service
- Downtime by service
- Recurrence rate and average tickets per incident
- Quarter-over-quarter trend arrows with comparison overlays
- Heatmap calendar and hour-of-day histogram
- Configurable metric cards
- All metrics computed in Rust for performance — single IPC call loads the entire dashboard
- Click any chart segment to drill down into the filtered incident list

### Report Generation
- One-click quarterly report generation in **DOCX** or **PDF** format
- 8 configurable sections: executive summary, metrics overview, incident timeline, P0/P1 breakdowns, service reliability, quarter-over-quarter comparison, discussion points, and action items
- Markdown content rendered as rich text in DOCX reports (bold, italic, lists, code blocks)
- Auto-generated discussion points based on 10 data-driven rules
- Dashboard charts embedded as PNG images (DOCX)
- Custom title and introduction with auto-narrative generation
- Report history tracking
- Save anywhere via native file dialog

### CSV Import
- Import incidents from Jira or other CSV exports
- Interactive column mapping with auto-detection
- Preview with validation (green/yellow/red row status)
- Saveable mapping templates for repeated imports
- Formula injection sanitization (OWASP-complete)

### Audit & Notifications
- Full audit trail for incident create/update/delete actions
- Activity feed on each incident showing change history
- Notification center with SLA breach alerts, overdue action items, and active incident counts
- Quarter-ending-soon reminders

### UX
- Collapsible sidebar with smooth transition
- Dark mode with system preference detection
- Command palette (Cmd+K) for quick navigation and search
- Keyboard shortcuts for all major actions
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
| Database | SQLite (sqlx, WAL mode) |
| Reports (DOCX) | docx-rs, pulldown-cmark |
| Reports (PDF) | genpdf |
| Forms | react-hook-form |
| Testing | vitest, @testing-library/react |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+N` | Quick-add incident |
| `Cmd+K` | Search incidents |
| `Cmd+S` | Save current form |
| `Cmd+1-4` | Navigate views (Dashboard, Incidents, Reports, Settings) |
| `/` | Focus search (when not in an input) |

## Getting Started

### Prerequisites

- **macOS** (Apple Silicon or Intel)
- **Rust** (via [rustup](https://rustup.rs/))
- **Node.js** 18+ and **pnpm**
- **Xcode Command Line Tools** (`xcode-select --install`)

### Install & Run

```bash
git clone https://github.com/saagar210/IncidentManagement.git
cd IncidentManagement
pnpm install
pnpm tauri dev
```

### Build for Production

```bash
pnpm tauri build
```

The `.dmg` installer will be in `src-tauri/target/release/bundle/dmg/`.

### Run Tests

```bash
# Rust tests (158 tests)
cd src-tauri && cargo test

# Frontend tests (16 tests)
pnpm test
```

## Project Structure

```
src/                          # React frontend
  components/                 # UI components
    ui/                       # shadcn/ui primitives (button, card, tabs, etc.)
    incidents/                # Incident-specific (SLA badge, activity feed, action items)
    dashboard/                # Charts, heatmap, metric cards
    layout/                   # App layout, sidebar, command palette, notifications
    onboarding/               # First-run wizard
  hooks/                      # TanStack Query hooks for all data operations
  views/                      # Page-level view components (7 views)
  types/                      # TypeScript type definitions
  lib/                        # Utilities (Tauri invoke wrapper, constants)
  test/                       # Test setup and mocks

src-tauri/src/                # Rust backend
  commands/                   # Tauri IPC command handlers (incidents, reports, SLA, audit)
  db/                         # SQLite initialization, 9 migrations, query modules
  models/                     # Data structs, validation, priority matrix, SLA, audit
  reports/                    # DOCX + PDF generation, markdown converter, 8 report sections
  import/                     # CSV parsing and column mapping
  security_tests.rs           # 158 security and correctness tests
```

## Database

SQLite with WAL mode. The database file is created automatically in the app data directory on first launch. Foreign keys are enforced per-connection via `SqliteConnectOptions`.

**Tables:** `incidents`, `services`, `action_items`, `quarter_config`, `import_templates`, `app_settings`, `tags`, `incident_tags`, `custom_field_definitions`, `custom_field_values`, `attachments`, `report_history`, `sla_definitions`, `audit_entries`

15 services, FY27 Q1-Q4 quarters, and P0-P4 SLA defaults are seeded on first run.

## License

MIT
