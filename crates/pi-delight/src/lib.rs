#![forbid(unsafe_code)]

//! Pure presentation helpers for shimmer, sparklines, fireworks, and titles.

use serde::{Deserialize, Serialize};

const BARS: [char; 8] = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShimmerMode {
    Cosine,
    Kitt,
}

#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn compute_shimmer_intensity(char_idx: usize, tick: u64, mode: ShimmerMode) -> f32 {
    let speed = 0.25;
    let width = 8.0;
    let phase = (tick as f32) * speed;
    match mode {
        ShimmerMode::Cosine => {
            let dist = ((char_idx as f32) - (phase % 40.0)).abs();
            if dist < width {
                (0.85 * (1.0 + (dist / width * std::f32::consts::PI).cos())).mul_add(0.5, 0.15)
            } else {
                0.15
            }
        }
        ShimmerMode::Kitt => {
            let cycle = 50.0;
            let ping_pong = ((phase % cycle) - (cycle / 2.0)).abs() * 2.0;
            let dist = ((char_idx as f32) - ping_pong).abs();
            if dist < width {
                (1.0 - (dist / width)).max(0.15)
            } else {
                0.15
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn render_sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let first = values
        .iter()
        .copied()
        .find(|value| value.is_finite())
        .unwrap_or(0.0);
    let mut min = first;
    let mut max = first;
    for &value in values {
        if value.is_finite() {
            min = min.min(value);
            max = max.max(value);
        }
    }
    let range = max - min;
    let mut out = String::with_capacity(values.len());
    for &value in values {
        let normalized = if range > 0.0 && value.is_finite() {
            (((value - min) / range * 7.0).round() as usize).min(7)
        } else {
            0
        };
        out.push(BARS[normalized]);
    }
    out
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: u32,
    pub max_life: u32,
    pub glyph: char,
}

#[derive(Debug, Clone, Default)]
pub struct FireworksState {
    pub particles: Vec<Particle>,
    pub frame_count: u32,
    pub is_active: bool,
}

impl FireworksState {
    #[allow(clippy::cast_precision_loss)]
    pub fn trigger_burst(&mut self, origin_x: f32, origin_y: f32, count: usize) {
        self.is_active = true;
        let glyphs = ['*', '✦', '✧', '•', '·', 'x'];
        for i in 0..count {
            let angle = (i as f32) / (count as f32) * 2.0 * std::f32::consts::PI;
            let speed = ((i % 3) as f32).mul_add(0.4, 1.2);
            self.particles.push(Particle {
                x: origin_x,
                y: origin_y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed * 0.6,
                life: 30,
                max_life: 30,
                glyph: glyphs[i % glyphs.len()],
            });
        }
    }

    pub fn tick(&mut self) {
        if !self.is_active {
            return;
        }
        self.frame_count += 1;
        for particle in &mut self.particles {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vy += 0.08;
            particle.life = particle.life.saturating_sub(1);
        }
        self.particles.retain(|particle| particle.life > 0);
        if self.particles.is_empty() {
            self.is_active = false;
        }
    }
}

#[must_use]
pub fn format_terminal_title(title: &str) -> String {
    let safe_title: String = title
        .chars()
        .filter(|character| !character.is_control())
        .take(256)
        .collect();
    format!("\x1b]0;{safe_title}\x07")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_helpers_keep_contracts() {
        assert!((0.0..=1.0).contains(&compute_shimmer_intensity(2, 4, ShimmerMode::Cosine)));
        assert_eq!(render_sparkline(&[1.0, 3.0, 5.0]).chars().count(), 3);
        assert_eq!(format_terminal_title("safe\x07name"), "\x1b]0;safename\x07");
    }

    #[test]
    fn fireworks_finish_after_thirty_ticks() {
        let mut fireworks = FireworksState::default();
        fireworks.trigger_burst(1.0, 2.0, 4);
        for _ in 0..30 {
            fireworks.tick();
        }
        assert!(!fireworks.is_active);
        assert!(fireworks.particles.is_empty());
    }
}
