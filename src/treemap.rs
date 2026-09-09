//! Balanced, deterministic treemap layout independent of the graphics backend.
#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
/// Nonzero weights only. Coordinates are normalized to [0,1]. At most 64 tiles.
pub fn layout(weights: &[u64]) -> Vec<Tile> {
    let items: Vec<_> = weights
        .iter()
        .enumerate()
        .take(64)
        .filter(|(_, w)| **w > 0)
        .map(|(i, w)| (i, *w as f64))
        .collect();
    let mut out = Vec::new();
    fn split(items: &[(usize, f64)], r: [f64; 4], out: &mut Vec<Tile>) {
        if items.is_empty() {
            return;
        }
        if items.len() == 1 {
            out.push(Tile {
                index: items[0].0,
                x: r[0] as f32,
                y: r[1] as f32,
                width: r[2] as f32,
                height: r[3] as f32,
            });
            return;
        }
        let total: f64 = items.iter().map(|v| v.1).sum();
        let mut left = items[0].1;
        let mut mid = 1;
        while mid < items.len() - 1
            && (left + items[mid].1 - total * 0.5).abs() < (left - total * 0.5).abs()
        {
            left += items[mid].1;
            mid += 1;
        }
        let ratio = left / total;
        if r[2] >= r[3] {
            split(&items[..mid], [r[0], r[1], r[2] * ratio, r[3]], out);
            split(
                &items[mid..],
                [r[0] + r[2] * ratio, r[1], r[2] * (1.0 - ratio), r[3]],
                out,
            );
        } else {
            split(&items[..mid], [r[0], r[1], r[2], r[3] * ratio], out);
            split(
                &items[mid..],
                [r[0], r[1] + r[3] * ratio, r[2], r[3] * (1.0 - ratio)],
                out,
            );
        }
    }
    split(&items, [0.0, 0.0, 1.0, 1.0], &mut out);
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_is_empty_and_extremes_are_finite() {
        assert!(layout(&[0, 0]).is_empty());
        for t in layout(&[u64::MAX, u64::MAX, 1]) {
            assert!(t.width.is_finite() && t.height.is_finite());
        }
    }
    #[test]
    fn rectangles_cover_without_overlapping() {
        let weights = [99, 81, 54, 32, 9, 2, 1];
        let tiles = layout(&weights);
        let sum: f64 = weights.iter().map(|n| *n as f64).sum();
        for (i, a) in tiles.iter().enumerate() {
            assert!(
                (a.width as f64 * a.height as f64 - weights[a.index] as f64 / sum).abs() < 1e-6
            );
            assert!(
                a.x >= 0.0 && a.y >= 0.0 && a.x + a.width <= 1.000001 && a.y + a.height <= 1.000001
            );
            for b in &tiles[i + 1..] {
                let w = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
                let h = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
                assert!(w <= 1e-6 || h <= 1e-6);
            }
        }
    }
    #[test]
    fn tile_count_is_bounded() {
        assert_eq!(layout(&[1; 200]).len(), 64);
    }
}
