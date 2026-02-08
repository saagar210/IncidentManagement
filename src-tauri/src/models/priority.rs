use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

impl Severity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Critical" => Some(Severity::Critical),
            "High" => Some(Severity::High),
            "Medium" => Some(Severity::Medium),
            "Low" => Some(Severity::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Impact {
    Critical,
    High,
    Medium,
    Low,
}

impl fmt::Display for Impact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Impact::Critical => write!(f, "Critical"),
            Impact::High => write!(f, "High"),
            Impact::Medium => write!(f, "Medium"),
            Impact::Low => write!(f, "Low"),
        }
    }
}

impl Impact {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Critical" => Some(Impact::Critical),
            "High" => Some(Impact::High),
            "Medium" => Some(Impact::Medium),
            "Low" => Some(Impact::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
            Priority::P4 => write!(f, "P4"),
        }
    }
}

pub fn calculate_priority(severity: &Severity, impact: &Impact) -> Priority {
    match (severity, impact) {
        (Severity::Critical, Impact::Critical) => Priority::P0,
        (Severity::Critical, Impact::High) => Priority::P1,
        (Severity::Critical, Impact::Medium) => Priority::P1,
        (Severity::Critical, Impact::Low) => Priority::P2,
        (Severity::High, Impact::Critical) => Priority::P1,
        (Severity::High, Impact::High) => Priority::P1,
        (Severity::High, Impact::Medium) => Priority::P2,
        (Severity::High, Impact::Low) => Priority::P3,
        (Severity::Medium, Impact::Critical) => Priority::P2,
        (Severity::Medium, Impact::High) => Priority::P2,
        (Severity::Medium, Impact::Medium) => Priority::P3,
        (Severity::Medium, Impact::Low) => Priority::P3,
        (Severity::Low, Impact::Critical) => Priority::P3,
        (Severity::Low, Impact::High) => Priority::P3,
        (Severity::Low, Impact::Medium) => Priority::P4,
        (Severity::Low, Impact::Low) => Priority::P4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_priority_combinations() {
        assert_eq!(calculate_priority(&Severity::Critical, &Impact::Critical), Priority::P0);
        assert_eq!(calculate_priority(&Severity::Critical, &Impact::High), Priority::P1);
        assert_eq!(calculate_priority(&Severity::Critical, &Impact::Medium), Priority::P1);
        assert_eq!(calculate_priority(&Severity::Critical, &Impact::Low), Priority::P2);
        assert_eq!(calculate_priority(&Severity::High, &Impact::Critical), Priority::P1);
        assert_eq!(calculate_priority(&Severity::High, &Impact::High), Priority::P1);
        assert_eq!(calculate_priority(&Severity::High, &Impact::Medium), Priority::P2);
        assert_eq!(calculate_priority(&Severity::High, &Impact::Low), Priority::P3);
        assert_eq!(calculate_priority(&Severity::Medium, &Impact::Critical), Priority::P2);
        assert_eq!(calculate_priority(&Severity::Medium, &Impact::High), Priority::P2);
        assert_eq!(calculate_priority(&Severity::Medium, &Impact::Medium), Priority::P3);
        assert_eq!(calculate_priority(&Severity::Medium, &Impact::Low), Priority::P3);
        assert_eq!(calculate_priority(&Severity::Low, &Impact::Critical), Priority::P3);
        assert_eq!(calculate_priority(&Severity::Low, &Impact::High), Priority::P3);
        assert_eq!(calculate_priority(&Severity::Low, &Impact::Medium), Priority::P4);
        assert_eq!(calculate_priority(&Severity::Low, &Impact::Low), Priority::P4);
    }
}
