pub mod discovery;
pub mod pipeline;
pub mod placement;
pub mod registry;

pub use discovery::{DiscoveredWidget, TEXT_WIDGETS_DIR, WIDGETS_DIR, scan_widgets};
pub use pipeline::{BuildOutput, TextCardOutcome, WidgetOutcome, build};
pub use placement::{Rect, SNAP_TOLERANCE, fits_canvas, overlaps, snap};
pub use registry::{WidgetGroup, WidgetSpec, all_widgets, find};
