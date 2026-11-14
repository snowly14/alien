macro_rules! pseudo_classes {
    ($($variant:ident => ($source:literal, $css:literal)),+ $(,)?) => {
        /// A CSS pseudo-class that takes no arguments.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(u8)]
        pub enum PseudoClass {
            $($variant),+
        }

        impl PseudoClass {
            const ALL: &[Self] = &[$(Self::$variant),+];

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($source => Some(Self::$variant),)+
                    _ => None,
                }
            }

            #[must_use]
            pub const fn as_css(self) -> &'static str {
                match self {
                    $(Self::$variant => $css),+
                }
            }
        }
    };
}

// Later pseudo-classes win ties in specificity at the same breakpoint.
pseudo_classes! {
    Root => ("root", ":root"),
    Empty => ("empty", ":empty"),
    FirstChild => ("firstChild", ":first-child"),
    LastChild => ("lastChild", ":last-child"),
    OnlyChild => ("onlyChild", ":only-child"),
    FirstOfType => ("firstOfType", ":first-of-type"),
    LastOfType => ("lastOfType", ":last-of-type"),
    OnlyOfType => ("onlyOfType", ":only-of-type"),
    Defined => ("defined", ":defined"),
    AnyLink => ("anyLink", ":any-link"),
    Link => ("link", ":link"),
    Visited => ("visited", ":visited"),
    Target => ("target", ":target"),
    Enabled => ("enabled", ":enabled"),
    Required => ("required", ":required"),
    Optional => ("optional", ":optional"),
    ReadOnly => ("readOnly", ":read-only"),
    ReadWrite => ("readWrite", ":read-write"),
    PlaceholderShown => ("placeholderShown", ":placeholder-shown"),
    Autofill => ("autofill", ":autofill"),
    Default => ("default", ":default"),
    Checked => ("checked", ":checked"),
    Indeterminate => ("indeterminate", ":indeterminate"),
    InRange => ("inRange", ":in-range"),
    OutOfRange => ("outOfRange", ":out-of-range"),
    Valid => ("valid", ":valid"),
    Invalid => ("invalid", ":invalid"),
    UserValid => ("userValid", ":user-valid"),
    UserInvalid => ("userInvalid", ":user-invalid"),
    Open => ("open", ":open"),
    Modal => ("modal", ":modal"),
    PopoverOpen => ("popoverOpen", ":popover-open"),
    Fullscreen => ("fullscreen", ":fullscreen"),
    Playing => ("playing", ":playing"),
    Paused => ("paused", ":paused"),
    Hover => ("hover", ":hover"),
    FocusWithin => ("focusWithin", ":focus-within"),
    Focus => ("focus", ":focus"),
    FocusVisible => ("focusVisible", ":focus-visible"),
    Active => ("active", ":active"),
    Disabled => ("disabled", ":disabled"),
}

const _: () = assert!(PseudoClass::ALL.len() <= u64::BITS as usize);

/// An unordered set of pseudo-classes that must all match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PseudoClasses(u64);

impl PseudoClasses {
    /// Adds a pseudo-class and reports whether it was new to this condition.
    pub fn insert(&mut self, pseudo_class: PseudoClass) -> bool {
        let bit = 1 << pseudo_class as u8;
        let was_new = self.0 & bit == 0;
        self.0 |= bit;
        was_new
    }

    #[must_use]
    pub const fn specificity(self) -> u32 {
        self.0.count_ones()
    }

    pub fn iter(self) -> impl Iterator<Item = PseudoClass> {
        let mut bits = self.0;
        std::iter::from_fn(move || {
            if bits == 0 {
                return None;
            }
            let index = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            Some(PseudoClass::ALL[index])
        })
    }
}
