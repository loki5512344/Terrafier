pub trait Brush: Send + Sync {
    fn get_strength(&self, dx: f64, dy: f64) -> f64;
    fn radius(&self) -> f64;
}

pub struct SymmetricBrush {
    pub radius: f64,
}

impl SymmetricBrush {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl Brush for SymmetricBrush {
    fn get_strength(&self, dx: f64, dy: f64) -> f64 {
        let dist = (dx * dx + dy * dy).sqrt();
        if dist >= self.radius {
            0.0
        } else {
            1.0 - (dist / self.radius)
        }
    }
    fn radius(&self) -> f64 {
        self.radius
    }
}
