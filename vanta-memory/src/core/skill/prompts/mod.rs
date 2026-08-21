//! Skill prompts (MEM-17, F4) — review agent + listing boilerplate.
//!
//! Rewritten in English from TDAM `MC/core/skill/prompts/*` (Principio 7).

pub mod skill_listing_prompt;
pub mod skill_review_prompt;

pub use skill_listing_prompt::{SKILLS_GUIDANCE, SKILL_LISTING_FOOTER, SKILL_LISTING_HEADER};
pub use skill_review_prompt::SKILL_REVIEW_PROMPT;
