// Privacy Guard, High-Speed Ad & Tracker Blocker, Permissions Manager, and Zero-Telemetry Audit
#![allow(unused_imports)]

pub mod adblock;
pub mod rules_engine;
pub mod permissions;
pub mod rule_parser;
pub mod audit;

pub use adblock::{AdBlockEngine, AdBlockStats};
pub use rules_engine::{FilterDecision, RuleCategory, RulesEngine};
pub use rule_parser::{ParsedRule, RuleParser};
pub use permissions::{PermissionKind, PermissionState, PermissionsManager, SitePermission};
pub use audit::{FlagAuditResult, ZeroTelemetryAuditor, MANDATED_PRIVACY_FLAGS, PROHIBITED_TELEMETRY_ENDPOINTS};
