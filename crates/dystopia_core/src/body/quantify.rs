use dystopia_derive::Quantified;
use num_enum::TryFromPrimitive;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
pub enum Temperature {
    Freezing,
    #[boundary(100.)]
    Cold,
    #[boundary(200.)]
    Habitable,
    #[boundary(400.)]
    Hot,
    #[boundary(500.)]
    Boiling,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
#[max(1.)]
pub enum Moisture {
    Parched,
    #[boundary(0.05)]
    Dry,
    #[boundary(0.15)]
    Moist,
    #[boundary(0.9)]
    Saturated,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
#[max(1.)]
pub enum Metallicity {
    Trace,
    #[boundary(0.05)]
    Low,
    #[boundary(0.2)]
    Moderate,
    #[boundary(0.4)]
    High,
    #[boundary(0.8)]
    Pure,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
pub enum Density {
    Diffuse,
    #[boundary(1.5)]
    LowDensity,
    #[boundary(2.3)]
    Rocky,
    #[boundary(3.5)]
    Dense,
    #[boundary(10.)]
    Degenerate,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
pub enum Illuminance {
    Faint,
    #[boundary(3000.)]
    Moderate,
    #[boundary(10000.)]
    Bright,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive, Quantified,
)]
#[repr(usize)]
#[quantify(f64)]
#[min(0.)]
pub enum AtmosphericDensity {
    Sparse,
    #[boundary(0.5)]
    Moderate,
    #[boundary(3.)]
    Thick,
}
