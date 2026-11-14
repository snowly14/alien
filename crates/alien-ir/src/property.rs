macro_rules! properties {
    ($($variant:ident => ($group:literal, $field:literal, $css:literal)),+ $(,)?) => {
        /// A CSS property supported by the compiler.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Property {
            $($variant),+
        }

        impl Property {
            #[must_use]
            pub fn from_name(group: &str, field: &str) -> Option<Self> {
                match (group, field) {
                    $(($group, $field) => Some(Self::$variant),)+
                    _ => None,
                }
            }

            #[must_use]
            pub const fn source_name(self) -> &'static str {
                match self {
                    $(Self::$variant => if $group.is_empty() { $field } else { concat!($group, ".", $field) }),+
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

properties! {
    Display => ("", "display", "display"),
    Position => ("", "position", "position"),
    BoxSizing => ("", "boxSizing", "box-sizing"),
    Width => ("", "width", "width"),
    Height => ("", "height", "height"),
    MinWidth => ("", "minWidth", "min-width"),
    MinHeight => ("", "minHeight", "min-height"),
    MaxWidth => ("", "maxWidth", "max-width"),
    MaxHeight => ("", "maxHeight", "max-height"),
    AspectRatio => ("", "aspectRatio", "aspect-ratio"),
    Inset => ("", "inset", "inset"),
    Top => ("", "top", "top"),
    Right => ("", "right", "right"),
    Bottom => ("", "bottom", "bottom"),
    Left => ("", "left", "left"),
    ZIndex => ("", "zIndex", "z-index"),
    Overflow => ("", "overflow", "overflow"),
    OverflowX => ("", "overflowX", "overflow-x"),
    OverflowY => ("", "overflowY", "overflow-y"),
    Flex => ("", "flex", "flex"),
    FlexDirection => ("", "flexDirection", "flex-direction"),
    FlexWrap => ("", "flexWrap", "flex-wrap"),
    FlexGrow => ("", "flexGrow", "flex-grow"),
    FlexShrink => ("", "flexShrink", "flex-shrink"),
    FlexBasis => ("", "flexBasis", "flex-basis"),
    Order => ("", "order", "order"),
    AlignItems => ("", "alignItems", "align-items"),
    AlignSelf => ("", "alignSelf", "align-self"),
    AlignContent => ("", "alignContent", "align-content"),
    JustifyContent => ("", "justifyContent", "justify-content"),
    JustifyItems => ("", "justifyItems", "justify-items"),
    JustifySelf => ("", "justifySelf", "justify-self"),
    GridTemplateColumns => ("", "gridTemplateColumns", "grid-template-columns"),
    GridTemplateRows => ("", "gridTemplateRows", "grid-template-rows"),
    GridAutoColumns => ("", "gridAutoColumns", "grid-auto-columns"),
    GridAutoRows => ("", "gridAutoRows", "grid-auto-rows"),
    GridAutoFlow => ("", "gridAutoFlow", "grid-auto-flow"),
    GridColumn => ("", "gridColumn", "grid-column"),
    GridRow => ("", "gridRow", "grid-row"),
    FontFamily => ("", "fontFamily", "font-family"),
    FontStyle => ("", "fontStyle", "font-style"),
    LetterSpacing => ("", "letterSpacing", "letter-spacing"),
    TextAlign => ("", "textAlign", "text-align"),
    TextDecorationLine => ("", "textDecorationLine", "text-decoration-line"),
    TextDecorationStyle => ("", "textDecorationStyle", "text-decoration-style"),
    TextOverflow => ("", "textOverflow", "text-overflow"),
    TextTransform => ("", "textTransform", "text-transform"),
    WhiteSpace => ("", "whiteSpace", "white-space"),
    OverflowWrap => ("", "overflowWrap", "overflow-wrap"),
    WordBreak => ("", "wordBreak", "word-break"),
    BackgroundRepeat => ("", "backgroundRepeat", "background-repeat"),
    BackgroundSize => ("", "backgroundSize", "background-size"),
    BackgroundPosition => ("", "backgroundPosition", "background-position"),
    Opacity => ("", "opacity", "opacity"),
    ObjectFit => ("", "objectFit", "object-fit"),
    ObjectPosition => ("", "objectPosition", "object-position"),
    OutlineStyle => ("", "outlineStyle", "outline-style"),
    OutlineWidth => ("", "outlineWidth", "outline-width"),
    OutlineOffset => ("", "outlineOffset", "outline-offset"),
    Visibility => ("", "visibility", "visibility"),
    Cursor => ("", "cursor", "cursor"),
    PointerEvents => ("", "pointerEvents", "pointer-events"),
    UserSelect => ("", "userSelect", "user-select"),
    Appearance => ("", "appearance", "appearance"),
    BackgroundColor => ("", "backgroundColor", "background-color"),
    Color => ("", "color", "color"),
    OutlineColor => ("", "outlineColor", "outline-color"),
    TextDecorationColor => ("", "textDecorationColor", "text-decoration-color"),
    Padding => ("", "padding", "padding"),
    PaddingTop => ("", "paddingTop", "padding-top"),
    PaddingRight => ("", "paddingRight", "padding-right"),
    PaddingBottom => ("", "paddingBottom", "padding-bottom"),
    PaddingLeft => ("", "paddingLeft", "padding-left"),
    PaddingBlock => ("", "paddingBlock", "padding-block"),
    PaddingInline => ("", "paddingInline", "padding-inline"),
    Margin => ("", "margin", "margin"),
    MarginTop => ("", "marginTop", "margin-top"),
    MarginRight => ("", "marginRight", "margin-right"),
    MarginBottom => ("", "marginBottom", "margin-bottom"),
    MarginLeft => ("", "marginLeft", "margin-left"),
    MarginBlock => ("", "marginBlock", "margin-block"),
    MarginInline => ("", "marginInline", "margin-inline"),
    Gap => ("", "gap", "gap"),
    RowGap => ("", "rowGap", "row-gap"),
    ColumnGap => ("", "columnGap", "column-gap"),
    BorderRadius => ("", "borderRadius", "border-radius"),
    BorderTopLeftRadius => ("", "borderTopLeftRadius", "border-top-left-radius"),
    BorderTopRightRadius => ("", "borderTopRightRadius", "border-top-right-radius"),
    BorderBottomRightRadius => ("", "borderBottomRightRadius", "border-bottom-right-radius"),
    BorderBottomLeftRadius => ("", "borderBottomLeftRadius", "border-bottom-left-radius"),
    BorderWidth => ("border", "width", "border-width"),
    BorderStyle => ("border", "style", "border-style"),
    BorderColor => ("border", "color", "border-color"),
    BorderTopWidth => ("borderTop", "width", "border-top-width"),
    BorderTopStyle => ("borderTop", "style", "border-top-style"),
    BorderTopColor => ("borderTop", "color", "border-top-color"),
    BorderRightWidth => ("borderRight", "width", "border-right-width"),
    BorderRightStyle => ("borderRight", "style", "border-right-style"),
    BorderRightColor => ("borderRight", "color", "border-right-color"),
    BorderBottomWidth => ("borderBottom", "width", "border-bottom-width"),
    BorderBottomStyle => ("borderBottom", "style", "border-bottom-style"),
    BorderBottomColor => ("borderBottom", "color", "border-bottom-color"),
    BorderLeftWidth => ("borderLeft", "width", "border-left-width"),
    BorderLeftStyle => ("borderLeft", "style", "border-left-style"),
    BorderLeftColor => ("borderLeft", "color", "border-left-color"),
    FontSize => ("", "fontSize", "font-size"),
    FontWeight => ("", "fontWeight", "font-weight"),
    LineHeight => ("", "lineHeight", "line-height"),
    BoxShadow => ("", "boxShadow", "box-shadow"),
    Translate => ("", "translate", "translate"),
    TransitionProperty => ("transition", "property", "transition-property"),
    TransitionDuration => ("transition", "duration", "transition-duration"),
    TransitionDelay => ("transition", "delay", "transition-delay"),
    TransitionEase => ("transition", "ease", "transition-timing-function"),
    AnimationName => ("animation", "name", "animation-name"),
    AnimationDuration => ("animation", "duration", "animation-duration"),
    AnimationDelay => ("animation", "delay", "animation-delay"),
    AnimationEase => ("animation", "ease", "animation-timing-function"),
    AnimationIterationCount => ("animation", "iterationCount", "animation-iteration-count"),
    AnimationDirection => ("animation", "direction", "animation-direction"),
    AnimationFillMode => ("animation", "fillMode", "animation-fill-mode"),
    AnimationPlayState => ("animation", "playState", "animation-play-state"),
}

impl Property {
    /// Reports whether the two properties can assign the same CSS longhand.
    #[must_use]
    pub fn conflicts_with(self, other: Self) -> bool {
        self == other || self.covers(other) || other.covers(self)
    }

    fn covers(self, other: Self) -> bool {
        use Property as P;

        matches!(
            (self, other),
            (
                P::Padding,
                P::PaddingTop
                    | P::PaddingRight
                    | P::PaddingBottom
                    | P::PaddingLeft
                    | P::PaddingBlock
                    | P::PaddingInline
            ) | (
                P::PaddingBlock | P::PaddingInline,
                P::PaddingTop | P::PaddingRight | P::PaddingBottom | P::PaddingLeft
            ) | (
                P::Margin,
                P::MarginTop
                    | P::MarginRight
                    | P::MarginBottom
                    | P::MarginLeft
                    | P::MarginBlock
                    | P::MarginInline
            ) | (
                P::MarginBlock | P::MarginInline,
                P::MarginTop | P::MarginRight | P::MarginBottom | P::MarginLeft
            ) | (P::Gap, P::RowGap | P::ColumnGap)
                | (P::Inset, P::Top | P::Right | P::Bottom | P::Left)
                | (P::Overflow, P::OverflowX | P::OverflowY)
                | (P::Flex, P::FlexGrow | P::FlexShrink | P::FlexBasis)
                | (
                    P::BorderRadius,
                    P::BorderTopLeftRadius
                        | P::BorderTopRightRadius
                        | P::BorderBottomRightRadius
                        | P::BorderBottomLeftRadius
                )
                | (
                    P::BorderWidth,
                    P::BorderTopWidth
                        | P::BorderRightWidth
                        | P::BorderBottomWidth
                        | P::BorderLeftWidth
                )
                | (
                    P::BorderStyle,
                    P::BorderTopStyle
                        | P::BorderRightStyle
                        | P::BorderBottomStyle
                        | P::BorderLeftStyle
                )
                | (
                    P::BorderColor,
                    P::BorderTopColor
                        | P::BorderRightColor
                        | P::BorderBottomColor
                        | P::BorderLeftColor
                )
        )
    }
}
