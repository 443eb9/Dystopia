pub enum Length {
    SolarRadius(f32),
    LightYear(f32),
    AstronomicalUnit(f32),
    Meter(f32),
}

pub enum Mass {
    SolarMass(f32),
    Kilogram(f32),
}

pub enum RadiantFlux {
    SolarLuminosity(f32),
    Watt(f32),
}

pub enum Temperature {
    Celsius(f32),
    Kelvin(f32),
}
