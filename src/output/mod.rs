mod json;
pub mod pager;
mod prompt;

pub use json::print_json;
pub use pager::page_text;
pub use prompt::confirm;
