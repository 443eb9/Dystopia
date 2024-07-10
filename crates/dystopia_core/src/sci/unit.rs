use dystopia_derive::Unit;

pub trait Unit {
    fn to_si(self) -> f64;
    fn to_si_unit(self) -> Self;
}

#[derive(Unit, Debug, Clone, Copy, PartialEq)]
pub enum Length {
    #[conversion = 695_700_000.]
    SolarRadius(f64),
    #[conversion = 9_460_730_472_580.8]
    LightYear(f64),
    #[conversion = 149_597_870_700.]
    AstronomicalUnit(f64),
    #[si]
    Meter(f64),
}

#[derive(Unit, Debug, Clone, Copy, PartialEq)]
pub enum Mass {
    #[conversion = 1.9884e30]
    SolarMass(f64),
    #[conversion = 5.972e24]
    EarthMass(f64),
    #[si]
    Kilogram(f64),
}

#[derive(Unit, Debug, Clone, Copy, PartialEq)]
pub enum RadiantFlux {
    #[conversion = 3.846e26]
    SolarLuminosity(f64),
    #[si]
    Watt(f64),
}

#[derive(Unit, Debug, Clone, Copy, PartialEq)]
pub enum Temperature {
    #[conversion = 273.15]
    #[conv_method = "sub"]
    Celsius(f64),
    #[si]
    Kelvin(f64),
}
