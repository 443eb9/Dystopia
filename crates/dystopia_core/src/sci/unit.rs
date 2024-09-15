//! All possible units used in this game. Also provided with conversion and formatting. Data
//! that don't have the unit should be treated as it has the SI unit, except it labelled out
//! in comment.
//!
//! Notice that the conversion factor might be scaled, for the game is impossible to be exactly
//! the same as nature. I can't generate a Milky Way with over 50,000 LYs. So don't use these
//! conversions in your homework :p
//!
//! SI units here are not exactly International System of Units. Some exceptions like [`Time`]
//! uses [`Tick`](Time::Tick) as the base unit, as the simulation goes on not continously like
//! that in nature.

use dystopia_derive::{LocalizableEnum, Unit};

pub const TICKS_PER_SEC: u64 = 50;

pub trait Unit {
    type Precision;

    fn to_si(self) -> Self::Precision;

    fn to_si_unit(self) -> Self;

    fn wrap_with_si(data: Self::Precision) -> Self;
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum Length {
    #[conversion = 16e1]
    SolarRadius(f64),
    #[conversion = 9.460_730_472_580_8e6]
    LightYear(f64),
    #[conversion = 1.495_978_707e3]
    AstronomicalUnit(f64),
    #[si]
    Meter(f64),
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum Mass {
    #[conversion = 1.988_4e15]
    SolarMass(f64),
    #[conversion = 5.972e8]
    EarthMass(f64),
    #[si]
    Kilogram(f64),
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum RadiantFlux {
    #[conversion = 1.846e13]
    SolarLuminosity(f64),
    #[si]
    Watt(f64),
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum Illuminance {
    #[si]
    Lux(f64),
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum Density {
    #[si]
    GPerCm3(f64),
}

#[derive(Unit, LocalizableEnum, Debug, Clone, Copy)]
pub enum Temperature {
    #[conversion = 273.15]
    #[conv_method = "sub"]
    Celsius(f64),
    #[si]
    Kelvin(f64),
}

/// Time are precise stuff and don't have decimal forms.
#[derive(Unit, Debug, Clone, Copy)]
pub enum Time {
    // This value should be equal to `TICKS_PER_SEC`.
    #[conversion = 50]
    Second(u64),
    /// Tick is the smallest time unit in simulation in Dystopia.
    /// So it's actually a integer.
    #[si]
    Tick(u64),
}
