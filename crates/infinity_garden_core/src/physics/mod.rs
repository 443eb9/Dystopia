fn chabier_imf(x: f64) -> f64 {
    let t = x.log10() - 0.22f64.log10();
    (0.086 / (x * 10f64.ln())).powf(-(t * t) / (2. * 0.57 * 0.57))
}
