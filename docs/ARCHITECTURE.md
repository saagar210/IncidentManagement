# Incident Management - System Architecture

## Overview

Incident Management is a **local-first, offline-capable desktop application** built with Tauri 2, React 19, and Rust. It enables IT teams to consolidate, analyze, and report on incidents without cloud dependencies.

**Core Values:**
- 🔒 Privacy-first: Data stays on your machine
- ⚡ Fast: Rust backend, optimized React UI
- 🔄 Offline-first: Works without internet
- 🤖 AI-ready: Optional Ollama integration for analysis

---

## System Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────┐
│                   USER INTERFACE                        │
│  React 19 + TypeScript + Tailwind CSS + shadcn/ui     │
│  (20 views, 50+ components, 23 custom hooks)          │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│              TAURI 2 IPC BRIDGE                         │
│  Command routing, JSON serialization, security layer   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         RUST BUSINESS LOGIC LAYER                       │
│  20 command modules, 160+ IPC handlers, validation     │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         DATABASE QUERY LAYER                            │
│  17 query modules, parameterized queries, caching      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│           SQLite Database (WAL Mode)                    │
│  15+ tables, 17 migrations, FTS5 full-text search     │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│         OPTIONAL: Ollama AI Integration                 │
│  Local LLM for analysis, trend detection, deduplication│
└─────────────────────────────────────────────────────────┘
```

---

## Technology Stack

### Frontend

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Framework | React | 19 | UI rendering |
| Language | TypeScript | 5.x | Type safety |
| Build Tool | Vite | 7 | Fast bundling |
| Routing | React Router | 7 | Navigation |
| State Management | TanStack Query | 5 | Server state, caching |
| Styling | Tailwind CSS | 4 | Utility-first CSS |
| Components | shadcn/ui | Latest | Accessible primitives |
| Forms | react-hook-form + Zod | Latest | Validation |
| Markdown | CodeMirror 6 + pulldown-cmark | Latest | Editing & rendering |
| Charts | Recharts 3 | Latest | Data visualization |
| Export | html2canvas | Latest | Chart export |

### Backend

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Language | Rust | 2021 edition | Memory safety, performance |
| Runtime | Tokio | 1.x | Async execution |
| Desktop Framework | Tauri | 2.10 | OS integration |
| Database | SQLite | 3.x | Persistent storage |
| DB Driver | sqlx | 0.8 | Async queries, type safety |
| Serialization | serde/serde_json | 1.x | JSON handling |
| Error Handling | thiserror | 2 | Error types |
| AI Client | ollama-rs | 0.3 | LLM integration |
| Reports | docx-rs, genpdf | 0.4, 0.2 | DOCX/PDF generation |
| Utilities | uuid, chrono | Latest | IDs, dates |

---

## Module Organization

### Frontend Structure

```
src/
├── views/                 # Page-level components (9 views)
│   ├── incidents-view     # Incident list + search
│   ├── incident-detail    # Single incident editing
│   ├── dashboard-view     # Metrics + analytics
│   ├── reports-view       # Report generation
│   ├── settings-view      # Configuration UI
│   ├── action-items       # Action tracking
│   ├── learnings          # Post-mortem database
│   ├── shift-handoff      # Shift transitions
│   └── trash              # Soft-deleted recovery
│
├── components/            # Reusable components (50+)
│   ├── ui/               # Base components (14: button, card, dialog, etc)
│   ├── layout/           # App shell (sidebar, navbar, etc)
│   ├── incidents/        # Incident-specific widgets
│   ├── dashboard/        # Dashboard visualizations
│   ├── ai/               # AI feature panels
│   └── import/           # CSV import wizard
│
├── hooks/                # Custom React hooks (23)
│   ├── use-incidents     # Incident data fetching
│   ├── use-services      # Service management
│   ├── use-dashboard     # Metrics calculation
│   ├── use-reports       # Report generation
│   ├── use-ai            # AI orchestration
│   └── ... (18 more)
│
├── types/                # TypeScript definitions
│   ├── incident.ts       # Incident, ActionItem models
│   ├── service.ts        # Service models
│   ├── report.ts         # Report structures
│   └── ... (more)
│
├── lib/                  # Utilities
│   ├── tauri-invoke      # IPC wrapper
│   ├── validators        # Zod validation schemas
│   ├── formatters        # Date/number formatting
│   └── constants         # Enums, defaults
│
└── test/                 # Test setup & mocks
```

### Backend Structure

```
src-tauri/src/
├── commands/             # Tauri IPC handlers (20 modules)
│   ├── incidents.rs      # Incident CRUD + search
│   ├── services.rs       # Service management
│   ├── reports.rs        # Report generation
│   ├── metrics.rs        # Dashboard calculations
│   ├── sla.rs            # SLA computation
│   ├── postmortems.rs    # Post-mortem workflow
│   ├── ai.rs             # AI orchestration
│   └── ... (13 more)
│
├── db/                   # Database layer
│   ├── mod.rs            # Pool initialization
│   ├── migrations.rs     # Migration runner
│   ├── sql/              # SQL files (17 migrations)
│   └── queries/          # Query modules (17)
│       ├── incidents.rs  # Incident queries
│       ├── services.rs   # Service queries
│       └── ... (15 more)
│
├── models/               # Domain models + validation
│   ├── incident.rs       # Incident, ActionItem
│   ├── service.rs        # Service
│   ├── request.rs        # Request DTOs
│   ├── response.rs       # Response DTOs
│   └── ... (validation methods)
│
├── ai/                   # AI integration (10 modules)
│   ├── client.rs         # Ollama HTTP client
│   ├── summarize.rs      # Summary generation
│   ├── root_cause.rs     # Root cause analysis
│   ├── similar.rs        # Similar incident detection
│   ├── dedup.rs          # Duplicate warnings
│   ├── trends.rs         # Service trends
│   └── ... (more)
│
├── reports/              # Report generation (4 modules)
│   ├── mod.rs            # Pipeline coordinator
│   ├── markdown.rs       # Markdown rendering
│   ├── pdf.rs            # PDF generation
│   └── charts.rs         # Chart export
│
├── error.rs              # Error types (9 variants)
├── lib.rs                # Tauri app initialization
├── main.rs               # Binary entry point
└── security_tests.rs     # 60+ security tests
```

---

## Data Model

### Core Entities

#### Incidents
- **Purpose:** Central entity representing a service disruption
- **Fields:** 50+ (title, severity, impact, timeline, analysis fields)
- **Relationships:** ← action_items, roles, tags, attachments, postmortems
- **Key Constraints:**
  - Severity: P0, P1, P2, P3, P4 (enforced via CHECK)
  - Status: active → acknowledged → monitoring → resolved → post-mortem
  - Impact: low, medium, high, critical
  - Priority: Auto-computed from severity × impact

#### Action Items
- **Purpose:** Follow-up tasks arising from incidents
- **Fields:** id, incident_id, title, status, due_date, assigned_to
- **Statuses:** open → in-progress → done
- **Constraints:** Due date must be ≥ created_at

#### Services
- **Purpose:** Service catalog for organizational structure
- **Fields:** id, name, tier (T1-T4), owner, runbook, dependencies
- **Relationships:** ← incidents (many), → service_dependencies (graph)

#### SLA Definitions
- **Purpose:** Service level targets by severity
- **Pre-seeded:**
  - P0: 15-min MTTA, 1-hour MTTR
  - P1: 30-min MTTA, 4-hour MTTR
  - P4: 1-day MTTA, 7-day MTTR

### Supporting Entities

| Entity | Purpose | Records |
|--------|---------|---------|
| tags | Incident categorization | M:N relationship |
| incident_roles | Team assignments | Multi-assignment per incident |
| custom_fields | Dynamic extensibility | EAV pattern |
| postmortems | RCA workflow | One per incident |
| checklists | Incident procedures | Templates + instances |
| audit_log | Change tracking | Immutable history |
| report_history | Output tracking | Generated reports |
| shift_handoffs | Transition reporting | Shift-specific |
| saved_filters | User preferences | Query presets |

---

## API Design

### Command Pattern (Tauri IPC)

All Rust functions exposed to frontend follow this pattern:

```rust
#[tauri::command]
pub async fn my_command(
    db: State<'_, SqlitePool>,
    request: MyRequest,
) -> Result<MyResponse, AppError> {
    // 1. Validate input
    request.validate()?;

    // 2. Execute business logic
    let result = db.query::something::do_thing(&request).await?;

    // 3. Audit log
    audit::insert_audit_entry(&db, ...).await.ok();

    // 4. Return response
    Ok(MyResponse { ... })
}
```

### Request/Response Types

**Naming Convention:**
- `Create<Entity>Request` - Input for POST
- `Update<Entity>Request` - Input for PUT
- `<Entity>Response` - Output from operations
- Types are separate from domain models

**Validation:**
- Implemented via `.validate()` method
- Returns `AppError::ValidationError` on failure
- Checked at command boundary before DB access

### Error Handling

**Error Types (9):**
1. `ValidationError` - Input validation failure (400)
2. `NotFoundError` - Resource doesn't exist (404)
3. `ConflictError` - Duplicate or state conflict (409)
4. `UnauthorizedError` - Permission denied (401)
5. `DatabaseError` - DB operation failure (500)
6. `AIError` - Ollama unavailable or error
7. `FileError` - File I/O problems
8. `InternalError` - Unexpected server error (500)
9. `Unknown` - Unclassified error

**Frontend Handling:**
```typescript
try {
  await invokeCommand("command", payload);
} catch (error: AppError) {
  switch (error.code) {
    case 'ValidationError':
      showFormInputError(error.message);
      break;
    case 'NotFoundError':
      showWarning('Item was deleted, refreshing...');
      break;
    default:
      showError('Operation failed. Please try again.');
  }
}
```

---

## State Management

### Frontend: TanStack Query (React Query)

**Pattern:**
```typescript
// Fetch data with caching
const { data, isLoading, error } = useQuery({
  queryKey: ['incidents', filters],
  queryFn: () => invokeCommand('list_incidents', filters),
  staleTime: 30000,  // Cache for 30s
});

// Mutations with side effects
const { mutate } = useMutation({
  mutationFn: (data) => invokeCommand('create_incident', data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: ['incidents'] });
  },
});
```

**Benefits:**
- Automatic cache management
- Background refetching
- Optimistic updates possible
- Request deduplication

### Backend: SQLite + Async Queries

**Concurrency Model:**
- Pool of async connections (Tokio)
- WAL mode enables concurrent reads
- Writes are serialized (SQLite limitation)
- Transactions for consistency

**Query Efficiency:**
- Parameterized queries (prevents SQL injection)
- Proper indexes on foreign keys + filters
- FTS5 for full-text search
- Lazy loading / pagination

---

## Security

### Input Validation

**All Request Types:**
```rust
pub trait Validate {
    fn validate(&self) -> Result<(), AppError>;
}
```

**Checked:**
- String length limits (title: 500, description: 10K)
- Required fields present
- Enum values in whitelist
- Date ranges logical (start ≤ end)
- UUID format valid

### SQL Injection Prevention

**Method:** Parameterized queries via sqlx
```rust
// ✅ SAFE: Parameter binding
sqlx::query("SELECT * FROM incidents WHERE id = ?")
    .bind(incident_id)
    .fetch_one(&pool)
    .await

// ❌ NEVER: String concatenation
sqlx::query(&format!("SELECT * FROM incidents WHERE id = '{}'", incident_id))
```

### CSV Injection Prevention

**Vulnerable:** CSV cells starting with `=`, `+`, `-`, `@`
**Defense:** Prefix suspicious values with single quote
```rust
let safe_value = if value.starts_with(['=', '+', '-', '@']) {
    format!("'{}'", value)
} else {
    value
};
```

### Audit Logging

**Logged Events:**
- Every CREATE operation
- Every UPDATE operation
- Every DELETE operation (soft delete tracked)
- Includes: actor, timestamp, old/new values

**Table:** `audit_log` (immutable)
```sql
CREATE TABLE audit_log (
    id UUID,
    timestamp DATETIME,
    entity_type TEXT,    -- 'incident', 'service', etc
    entity_id TEXT,      -- Specific record ID
    operation TEXT,      -- 'created', 'updated', 'deleted'
    actor TEXT,          -- User or 'system'
    old_values JSON,     -- Before state
    new_values JSON,     -- After state
    changes JSON         -- { field: { old, new } }
);
```

---

## Performance Characteristics

### Benchmarks (Target)

| Operation | Target | Notes |
|-----------|--------|-------|
| List 1000 incidents | <500ms | Paginated, cached |
| Search (FTS5) | <100ms | Full-text index |
| Create incident | <50ms | Async writes |
| Generate report | <10s | DOCX with charts |
| Dashboard metrics | <200ms | Computed in Rust |

### Optimization Strategies

1. **Frontend:**
   - Code splitting: vendor chunks (markdown, charts, tauri)
   - Tree-shaking: unused code removed
   - Lazy loading: views load on demand
   - TanStack Query: cache prevents re-fetches

2. **Backend:**
   - Async/await: Tokio handles concurrency
   - Indexes: FKs and filters indexed
   - FTS5: Full-text search optimized
   - Computed columns: MTTR, duration calculated

3. **Database:**
   - WAL mode: Concurrent reads while writing
   - Prepared statements: Compiled SQL
   - VACUUM: Periodically optimize
   - Statistics: Query planner hints

---

## Extensibility

### Custom Fields (EAV Pattern)

Users can add dynamic fields to incidents:

```typescript
// UI adds field definition
await invokeCommand('create_custom_field', {
  name: 'Affected Regions',
  type: 'select',
  options: ['US', 'EU', 'APAC'],
});

// Values stored separately
await invokeCommand('set_incident_custom_fields', {
  incident_id: 'inc-123',
  fields: { 'Affected Regions': 'EU' },
});
```

### Checklist Templates

Reusable incident procedures:
```typescript
// Define template
await invokeCommand('create_checklist_template', {
  name: 'Database Failover',
  items: [
    'Verify failover triggered',
    'Check replica lag',
    'Confirm all services reconnected',
  ],
});

// Use on incident
await invokeCommand('create_incident_checklist', {
  incident_id: 'inc-456',
  template_id: 'tmpl-db-failover',
});
```

---

## Deployment

### Distribution

- **Platform Support:**
  - macOS (Intel & Apple Silicon)
  - Linux (AppImage, .deb)
  - Windows (MSI)

- **Binary Size:** ~2-3MB (compressed Tauri bundle)

- **Database Location:**
  - macOS: `~/.config/incident-manager/`
  - Linux: `~/.config/incident-manager/`
  - Windows: `%AppData%\incident-manager\`

### Future Enhancements

1. **Cloud Deployment (Post-MVP):**
   - Replace Tauri with web server (same Rust backend)
   - Add authentication layer (JWT)
   - Implement multi-tenant isolation
   - Cloud database (PostgreSQL)

2. **Mobile (Post-MVP):**
   - React Native for iOS/Android
   - Same backend API
   - Offline-first with sync

---

## Testing

### Test Coverage

| Layer | Type | Count | Coverage |
|-------|------|-------|----------|
| Frontend Hooks | Unit | 19 | 80%+ |
| Backend Commands | Unit | 30 | 85%+ |
| Integration | E2E | 5 | Critical paths |
| Security | Compliance | 60+ | OWASP, FedRAMP |

### Running Tests

```bash
# Frontend tests
pnpm test:run

# Backend tests (requires GTK)
cd src-tauri && cargo test --lib

# E2E integration tests
cd src-tauri && cargo test --test '*'

# Check coverage
pnpm test:coverage
```

---

## References

- [Tauri Documentation](https://tauri.app)
- [React Documentation](https://react.dev)
- [Rust Book](https://doc.rust-lang.org/book/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

**Document Version:** 1.0
**Last Updated:** February 2025
**Status:** Production
