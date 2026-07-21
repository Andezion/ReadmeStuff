pub mod pipeline;
pub mod registry;

pub use pipeline::{BuildOutput, WidgetOutcome, build};
pub use registry::{WidgetGroup, WidgetSpec, all_widgets, find};
