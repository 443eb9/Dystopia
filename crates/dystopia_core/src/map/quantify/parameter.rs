#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Temperature {
    /// 0-100 K
    Freezing,
    /// 100-200 K
    Cold,
    /// 200-400 K
    Habitable,
    /// 400-500 K
    Hot,
    /// 500-inf K
    Boiling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Moisture {
    /// 0-5 %
    Parched,
    /// 5-15 %
    Dry,
    /// 15-90 %
    Moist,
    /// 90-100 %
    Saturated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metallicity {
    /// 0-5 %
    Trace,
    /// 5-20 %
    Low,
    /// 20-40 %
    Moderate,
    /// 40-80 %
    High,
    /// 80-100 %
    Pure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Density {
    /// 0-1.5 g/cm^3
    Diffuse,
    /// 1.5-2.3 g/cm^3
    LowDensity,
    /// 2.3-3.5 g/cm^3
    Rocky,
    /// 3.5-4.5 g/cm^3
    Dense,
    /// 4.5+ g/cm^3
    Degenerate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightIntensity {
    Faint,
    Moderate,
    Bright,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtmosphericDensity {
    /// 0-0.5 g/cm^3
    Sparse,
    /// 0.5-3 g/cm^3
    Moderate,
    /// 3+ g/cm^3
    Thick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biomass {
    Minimal,
    Low,
    Moderate,
    Abundant,
}
