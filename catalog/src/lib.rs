pub mod pipeline;
pub mod registry;

pub use pipeline::{BuildOutput, TextCardOutcome, WidgetOutcome, build};
pub use registry::{WidgetGroup, WidgetSpec, all_widgets, find};
