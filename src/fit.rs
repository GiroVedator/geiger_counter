use polyfit::MonomialFit;

pub struct Fit{
    coeffs: Vec<f64>,
}

impl Fit {
    pub fn new(x: &[f64], y: &[f64], degree: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let data: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)).collect();
        let fit = MonomialFit::new(&data, degree)?;
        Ok(Fit { coeffs: fit.coefficients().to_vec() })
    }

    pub fn predict(&self, x: f64) -> f64 {
        self.coeffs.iter().enumerate().map(|(i, coeff)| 1000.0*coeff * x.powi(i as i32)).sum()
    }
}