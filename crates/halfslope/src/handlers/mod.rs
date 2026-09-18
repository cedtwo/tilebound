//! [`DisplaceHandler`](tilebound::DisplaceHandler) implementations.
mod vertex_handler;

mod attach;
mod slide;

pub use attach::Attach;
pub use slide::Slide;
pub(super) use vertex_handler::VertexHandler;
