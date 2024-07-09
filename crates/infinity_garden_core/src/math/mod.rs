use std::ops::Range;

use rand::Rng;

fn reject_sampling(
    rng: &mut impl Rng,
    pdf: impl Fn(f64) -> f64,
    x_range: Range<f64>,
    y_range: Range<f64>,
    num_samples: u32,
    batch_size: u32,
) -> Vec<f64> {
    let num_samples = num_samples as usize;
    let mut samples = Vec::with_capacity(num_samples as usize);

    while samples.len() < num_samples {
        let x = (0..batch_size)
            .map(|_| rng.gen_range(x_range.clone()))
            .collect::<Vec<_>>();
        let y = (0..batch_size)
            .map(|_| rng.gen_range(y_range.clone()))
            .collect::<Vec<_>>();

        let new_samples = x
            .into_iter()
            .zip(y)
            .filter_map(|(x, y)| if y < pdf(x) { Some(x) } else { None });
        samples.extend(new_samples);
    }

    samples
}
