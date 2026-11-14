#![doc = env!("CARGO_PKG_DESCRIPTION")]

mod border;
mod color;
mod condition;
mod declaration;
mod length;
mod property;
mod style;

pub use border::{BorderStyle, BorderWidth};
pub use color::{ColorKeyword, ColorToken, ColorValue};
pub use condition::{Condition, PseudoClass, PseudoClasses};
pub use declaration::{Declaration, ListSeparator, TokenFamily, Value};
pub use length::{Length, LengthUnit};
pub use property::Property;
pub use style::{Rule, Style};

mod config;
pub use config::{GlobalConfig, Mode, Reset};
