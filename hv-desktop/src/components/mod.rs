//! Reusable UI components per PRODUCT_BLUEPRINT.md Section 0.9
//!
//! The left column is assembled from the `column_*` family, which shares one
//! answer to "how wide am I?" (`column_density`), so all six of its sections
//! change shape at exactly the same breakpoints.

pub mod channel_meter;
pub mod collapsible_section;
pub mod column_controls;
pub mod column_density;
pub mod dropdown_with_meta;
pub mod empty_state;
pub mod meter_bar;
pub mod panel_chrome;
pub mod panel_section;
pub mod slider_with_readout;
pub mod status_pill;
pub mod toggle_chip;

pub use column_density::ColumnDensity;
pub use meter_bar::MeterBar;
pub use panel_section::{PanelSection, SectionStatus};
