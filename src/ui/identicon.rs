use egui::{Color32, Pos2, Rect, Vec2};

/// Generates a unique blocky identicon based on a string hash
pub struct Identicon;

impl Identicon {
    const GRID_SIZE: usize = 5;

    // Vibrant color palette for identicons
    const COLORS: &'static [Color32] = &[
        Color32::from_rgb(239, 68, 68),  // Red
        Color32::from_rgb(249, 115, 22), // Orange
        Color32::from_rgb(245, 158, 11), // Amber
        Color32::from_rgb(132, 204, 22), // Lime
        Color32::from_rgb(34, 197, 94),  // Green
        Color32::from_rgb(20, 184, 166), // Teal
        Color32::from_rgb(6, 182, 212),  // Cyan
        Color32::from_rgb(59, 130, 246), // Blue
        Color32::from_rgb(99, 102, 241), // Indigo
        Color32::from_rgb(139, 92, 246), // Purple
        Color32::from_rgb(168, 85, 247), // Violet
        Color32::from_rgb(236, 72, 153), // Pink
    ];

    /// Simple hash function for strings
    fn hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }

    /// Generate a grid pattern from a hash
    fn generate_grid(hash: u64) -> [[bool; Self::GRID_SIZE]; Self::GRID_SIZE] {
        let mut grid = [[false; Self::GRID_SIZE]; Self::GRID_SIZE];

        // Generate left half + center column (symmetric)
        let half = Self::GRID_SIZE.div_ceil(2);
        for (y, row) in grid.iter_mut().enumerate() {
            for x in 0..half {
                let bit_index = y * half + x;
                let bit = (hash >> (bit_index % 64)) & 1;
                row[x] = bit == 1;
                // Mirror to right side
                row[Self::GRID_SIZE - 1 - x] = bit == 1;
            }
        }

        grid
    }

    /// Get color from hash
    fn get_color(hash: u64) -> Color32 {
        let index = (hash % Self::COLORS.len() as u64) as usize;
        Self::COLORS[index]
    }

    /// Draw the identicon
    pub fn draw(ui: &mut egui::Ui, id: &str, size: f32) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let hash = Self::hash(id);
            let grid = Self::generate_grid(hash);
            let color = Self::get_color(hash);
            let bg_color = Color32::from_rgb(28, 28, 36);

            // Draw background with rounded corners
            painter.rect_filled(rect, 8.0, bg_color);

            let padding = size * 0.15;
            let inner_size = size - padding * 2.0;
            let cell_size = inner_size / Self::GRID_SIZE as f32;

            for (y, row) in grid.iter().enumerate() {
                for (x, &filled) in row.iter().enumerate() {
                    if filled {
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(
                                rect.min.x + padding + x as f32 * cell_size,
                                rect.min.y + padding + y as f32 * cell_size,
                            ),
                            Vec2::splat(cell_size * 0.9),
                        );
                        painter.rect_filled(cell_rect, 2.0, color);
                    }
                }
            }
        }

        response
    }

    /// Draw a small status indicator dot
    pub fn draw_status_dot(ui: &mut egui::Ui, connected: bool) {
        let size = 8.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let color = if connected {
                Color32::from_rgb(34, 197, 94) // Green
            } else {
                Color32::from_rgb(100, 100, 115) // Gray
            };
            ui.painter().circle_filled(rect.center(), size / 2.0, color);
        }
    }
}
