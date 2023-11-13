pub trait MapRange {
    fn map(&self, src: (f32, f32), dst: (f32, f32)) -> f32;
}

impl MapRange for f32 {
    fn map(&self, src: (f32, f32), dst: (f32, f32)) -> f32 {
        if src.0 == src.1 {
            return dst.0; // avoid div by 0
        }
        let m = (dst.1 - dst.0) / (src.1 - src.0);
        let b = ((dst.0*src.1) - (dst.1*src.0)) / (src.1 - src.0);
        // y = mx+b
        (self * m) + b
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_functionality() {
        assert_eq!(2.5.map((0.0, 10.0), (0.0, 100.0)), 25.0);
    }

    #[test]
    fn edge_cases() {
        assert_eq!(0.0.map((0.0, 10.0), (0.0, 100.0)), 0.0);
        assert_eq!(10.0.map((0.0, 10.0), (0.0, 100.0)), 100.0);
    }

    #[test]
    fn inverted_source_range() {
        assert_eq!(2.5.map((10.0, 0.0), (0.0, 100.0)), 75.0);
    }

    #[test]
    fn inverted_destination_range() {
        assert_eq!(2.5.map((0.0, 10.0), (100.0, 0.0)), 75.0);
    }

    #[test]
    fn zero_range() {
        assert_eq!(5.0.map((5.0, 5.0), (0.0, 100.0)), 0.0);
        assert_eq!(5.0.map((0.0, 10.0), (50.0, 50.0)), 50.0);
    }

    #[test]
    fn negative_values() {
        assert_eq!((-2.5).map((-10.0, 0.0), (0.0, 100.0)), 75.0);
    }

    #[test]
    fn large_values() {
        assert_eq!(1e9.map((0.0, 1e10), (0.0, 100.0)), 10.0);
        assert_eq!(5e9.map((0.0, 1e10), (0.0, 100.0)), 50.0);
    }
}

