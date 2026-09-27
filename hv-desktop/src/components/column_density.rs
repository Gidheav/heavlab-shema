//! ColumnDensity — how the left column answers the question "how wide am I?".
//!
//! The layout owns the answer. Every section asks this module instead of
//! measuring pixels itself, so all six sections change shape at exactly the
//! same breakpoints and the column reads as one surface rather than six
//! widgets that happen to share a parent.

/// Below this the column cannot fit a label legibly, so it goes icon-only.
pub const ICON_ONLY_BELOW: f32 = 200.0;
/// At or above this there is room for secondary readouts and metadata.
pub const EXPANDED_FROM: f32 = 400.0;

/// Everything scales off this. Padding, gaps, and control heights are all
/// multiples so the column keeps its rhythm at any zoom level.
pub const UNIT: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDensity {
    /// Labels are gone, icons are centred, and section bodies are suppressed.
    IconOnly,
    /// Labels and primary readouts.
    Compact,
    /// Everything, plus secondary readouts and inline metadata.
    Expanded,
}

impl ColumnDensity {
    /// Resolves the density for a given available width. Widths below
    /// [`ICON_ONLY_BELOW`] never fall through to `Compact`, so a squeezed
    /// column degrades all the way to icons instead of half-fitting labels.
    pub fn from_width(width: f32) -> Self {
        if width < ICON_ONLY_BELOW {
            Self::IconOnly
        } else if width >= EXPANDED_FROM {
            Self::Expanded
        } else {
            Self::Compact
        }
    }

    /// Whether words may be drawn at all.
    pub fn shows_labels(self) -> bool {
        self != Self::IconOnly
    }

    /// Whether secondary values and metadata may be drawn.
    pub fn shows_secondary(self) -> bool {
        self == Self::Expanded
    }

    /// Whether a section body renders at all. Icon-only headers only.
    pub fn shows_bodies(self) -> bool {
        self != Self::IconOnly
    }

    /// Horizontal padding inside a section body.
    pub fn pad_x(self) -> f32 {
        match self {
            Self::IconOnly => 0.0,
            Self::Compact => UNIT * 2.0,
            Self::Expanded => UNIT * 3.0,
        }
    }

    /// Gap between stacked rows inside a body. Whole units only, so the column
    /// keeps to the 4 px grid at every width.
    pub fn row_gap(self) -> f32 {
        match self {
            Self::IconOnly => 0.0,
            Self::Compact => UNIT,
            Self::Expanded => UNIT * 2.0,
        }
    }

    /// Height of a one-line control row.
    pub fn row_height(self) -> f32 {
        match self {
            Self::IconOnly => UNIT * 4.0,
            Self::Compact => UNIT * 4.0,
            Self::Expanded => UNIT * 5.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ColumnDensity, EXPANDED_FROM, ICON_ONLY_BELOW};

    #[test]
    fn breakpoints_match_the_specification() {
        assert_eq!(ColumnDensity::from_width(199.0), ColumnDensity::IconOnly);
        assert_eq!(ColumnDensity::from_width(ICON_ONLY_BELOW), ColumnDensity::Compact);
        assert_eq!(ColumnDensity::from_width(300.0), ColumnDensity::Compact);
        assert_eq!(
            ColumnDensity::from_width(EXPANDED_FROM - 1.0),
            ColumnDensity::Compact
        );
        assert_eq!(ColumnDensity::from_width(EXPANDED_FROM), ColumnDensity::Expanded);
        assert_eq!(ColumnDensity::from_width(640.0), ColumnDensity::Expanded);
    }

    #[test]
    fn a_squeezed_column_never_half_fits_labels() {
        // A very narrow window must not land in IconOnly half-way — the
        // operator would see titles with no content under them.
        for width in [0.0, 40.0, 120.0, ICON_ONLY_BELOW - 1.0] {
            assert_eq!(ColumnDensity::from_width(width), ColumnDensity::IconOnly);
        }
    }

    #[test]
    fn every_padding_and_gap_is_a_multiple_of_the_base_unit() {
        for density in [
            ColumnDensity::IconOnly,
            ColumnDensity::Compact,
            ColumnDensity::Expanded,
        ] {
            for value in [density.pad_x(), density.row_gap(), density.row_height()] {
                let units = value / super::UNIT;
                assert!(
                    (units - units.round()).abs() < f32::EPSILON,
                    "{value} is not a multiple of the 4px base unit"
                );
            }
        }
    }

    #[test]
    fn wider_columns_never_shrink() {
        let widths = [200.0, 300.0, 400.0, 600.0];
        for pair in widths.windows(2) {
            let narrow = ColumnDensity::from_width(pair[0]);
            let wide = ColumnDensity::from_width(pair[1]);
            assert!(
                wide.pad_x() >= narrow.pad_x() && wide.row_gap() >= narrow.row_gap(),
                "density must not lose room as the column grows"
            );
        }
    }

    #[test]
    fn icon_only_suppresses_every_label_and_body() {
        let icon_only = ColumnDensity::IconOnly;
        assert!(!icon_only.shows_labels());
        assert!(!icon_only.shows_secondary());
        assert!(!icon_only.shows_bodies());

        let compact = ColumnDensity::Compact;
        assert!(compact.shows_labels());
        assert!(!compact.shows_secondary());
        assert!(compact.shows_bodies());

        let expanded = ColumnDensity::Expanded;
        assert!(expanded.shows_labels());
        assert!(expanded.shows_secondary());
        assert!(expanded.shows_bodies());
    }
}
