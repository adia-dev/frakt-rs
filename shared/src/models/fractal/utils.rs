pub fn convergence_value(pzn: f64, threshold: f64, count: u32, nmax: u32) -> f64 {
    let accuracy = f64::log10(threshold);
    if count < nmax {
        0.5 - 0.5 * f64::cos(0.1 * (count as f64 - (f64::log10(pzn) / accuracy)))
    } else {
        1.0
    }
}
