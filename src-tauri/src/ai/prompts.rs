pub fn summarize_system() -> &'static str {
    "You are an incident management expert. Generate concise, professional incident summaries suitable for executive briefings. Focus on impact, timeline, and resolution."
}

pub fn summarize_prompt(
    title: &str,
    severity: &str,
    status: &str,
    service: &str,
    root_cause: &str,
    resolution: &str,
    notes: &str,
) -> String {
    format!(
        "Summarize this incident for an executive audience:\n\n\
        Title: {}\n\
        Severity: {}\n\
        Status: {}\n\
        Service: {}\n\
        Root Cause: {}\n\
        Resolution: {}\n\
        Notes: {}\n\n\
        Provide a 2-3 paragraph executive summary covering impact, root cause, and current status/resolution.",
        title,
        severity,
        status,
        service,
        if root_cause.is_empty() {
            "Not yet determined"
        } else {
            root_cause
        },
        if resolution.is_empty() {
            "In progress"
        } else {
            resolution
        },
        if notes.is_empty() { "None" } else { notes }
    )
}

pub fn stakeholder_system() -> &'static str {
    "You are drafting a stakeholder communication about an incident. Be professional, clear, and avoid jargon. Include what happened, current status, impact, and next steps."
}

pub fn stakeholder_prompt(
    title: &str,
    severity: &str,
    status: &str,
    service: &str,
    impact: &str,
    notes: &str,
) -> String {
    format!(
        "Draft a stakeholder update email for this incident:\n\n\
        Title: {}\n\
        Severity: {}\n\
        Status: {}\n\
        Service: {}\n\
        Impact Level: {}\n\
        Notes: {}\n\n\
        Write a professional, empathetic update suitable for sending to affected stakeholders.",
        title,
        severity,
        status,
        service,
        impact,
        if notes.is_empty() { "None" } else { notes }
    )
}

pub fn postmortem_system() -> &'static str {
    "You are an expert in writing post-mortem documents for production incidents. Create thorough, blameless post-mortems that focus on systems improvements."
}

pub fn postmortem_prompt(
    title: &str,
    severity: &str,
    service: &str,
    root_cause: &str,
    resolution: &str,
    lessons: &str,
    contributing_factors: &[String],
) -> String {
    let factors = if contributing_factors.is_empty() {
        "None documented".to_string()
    } else {
        contributing_factors.join("\n- ")
    };

    format!(
        "Generate a comprehensive post-mortem document for this incident:\n\n\
        Title: {}\n\
        Severity: {}\n\
        Service: {}\n\
        Root Cause: {}\n\
        Resolution: {}\n\
        Lessons Learned: {}\n\
        Contributing Factors:\n- {}\n\n\
        Structure the post-mortem with these sections:\n\
        1. Executive Summary\n\
        2. Impact Analysis\n\
        3. Timeline\n\
        4. Root Cause Analysis\n\
        5. Contributing Factors\n\
        6. Action Items\n\
        7. Lessons Learned\n\n\
        Use blameless language. Focus on system improvements.",
        title,
        severity,
        service,
        if root_cause.is_empty() {
            "Not yet determined"
        } else {
            root_cause
        },
        if resolution.is_empty() {
            "In progress"
        } else {
            resolution
        },
        if lessons.is_empty() {
            "None documented"
        } else {
            lessons
        },
        factors
    )
}
