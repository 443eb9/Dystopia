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

use dystopia_derive::Unit;

pub trait Unit<U> {
    fn to_si(self) -> U;
    fn to_si_unit(self) -> Self;
}

#[derive(Unit, Debug, Clone, Copy)]
#[precision(f64)]
pub enum Length {
    // #[conversion = 695_000_000.]
    #[conversion = 695.]
    SolarRadius(f64),
    #[conversion = 9_460_730_472_580.8]
    LightYear(f64),
    #[conversion = 149_597_870_700.]
    AstronomicalUnit(f64),
    #[si]
    Meter(f64),
    /// The "meter" used for rendering. It scales the appearance of bodies,
    /// but won't affect their actual sizes.
    #[conversion = 1e-6]
    RenderMeter(f64),
}

#[derive(Unit, Debug, Clone, Copy)]
#[precision(f64)]
pub enum Mass {
    #[conversion = 1.988_4e30]
    SolarMass(f64),
    #[conversion = 5.972e24]
    EarthMass(f64),
    #[si]
    Kilogram(f64),
}

#[derive(Unit, Debug, Clone, Copy)]
#[precision(f64)]
pub enum RadiantFlux {
    #[conversion = 3.846e26]
    SolarLuminosity(f64),
    #[si]
    Watt(f64),
}

#[derive(Unit, Debug, Clone, Copy)]
#[precision(f64)]
pub enum Temperature {
    #[conversion = 273.15]
    #[conv_method = "sub"]
    Celsius(f64),
    #[si]
    Kelvin(f64),
}

/// Time are precise stuff and don't have decimal forms.
#[derive(Unit, Debug, Clone, Copy)]
#[precision(u64)]
pub enum Time {
    #[conversion = 50]
    Second(u64),
    /// Tick is the smallest time unit in simulation in Dystopia.
    /// So it's actually a integer.
    #[si]
    Tick(u64),
}
