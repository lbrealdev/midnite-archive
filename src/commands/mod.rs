mod comments;
mod download;
mod generate;
mod rename;

pub use comments::execute as comments;
pub use download::execute as download;
pub use generate::execute as generate;
pub use rename::execute as rename;
