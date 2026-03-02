use noise::{NoiseFn, Perlin};

pub fn octave_noise_2d(x: f64, y: f64, ocatves: usize, persistence: f64, perlin: &Perlin) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max = 0.0;

    for _ in 0..ocatves {
        total += perlin.get([x * frequency, y * frequency]) * amplitude;
        max += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }

    total / max
}