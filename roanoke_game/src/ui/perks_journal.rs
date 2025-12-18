//! Journal UI System
//!
//! A book-style journal with three main sections:
//! - Perks: Unlockable abilities across perk trees
//! - Stats: Player statistics and progression
//! - Encyclopedia: Discovered fauna, flora, and lore
//!
//! Features:
//! - Main tabs across the top (Perks, Stats, Encyclopedia)
//! - Left-side emblem tabs for categories within each section
//! - Inner tabs for sub-navigation
//! - Artwork display for perk details

use egui::{Color32, TextureHandle, TextureId};
use std::collections::HashMap;

// ============================================================================
// MAIN JOURNAL TABS
// ============================================================================

/// Top-level journal sections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JournalSection {
    #[default]
    Perks,
    Stats,
    Encyclopedia,
    Settings,
}

impl JournalSection {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Perks => "Perks",
            Self::Stats => "Stats",
            Self::Encyclopedia => "Encyclopedia",
            Self::Settings => "Settings",
        }
    }

    pub fn all() -> &'static [JournalSection] {
        &[Self::Perks, Self::Stats, Self::Encyclopedia, Self::Settings]
    }
}

// ============================================================================
// NATURALIST CATEGORIES (Left tabs when in Perks section)
// ============================================================================

/// Naturalist skill categories - displayed as emblem tabs on the left
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PerkTree {
    #[default]
    Hunting,
    Fishing,
    Horseback,
    Husbandry,
    Mining,
    Foraging,
    Woodcutting,
}

impl PerkTree {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hunting => "Hunting",
            Self::Fishing => "Fishing",
            Self::Horseback => "Horseback",
            Self::Husbandry => "Husbandry",
            Self::Mining => "Mining",
            Self::Foraging => "Foraging",
            Self::Woodcutting => "Woodcutting",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Hunting => "Track and harvest wild game",
            Self::Fishing => "Master the rivers and coastline",
            Self::Horseback => "Bond with and train horses",
            Self::Husbandry => "Raise hounds, fowl, and livestock",
            Self::Mining => "Extract ore and precious stones",
            Self::Foraging => "Gather herbs, berries, and roots",
            Self::Woodcutting => "Fell trees and process lumber",
        }
    }

    pub fn emblem_path(&self) -> &'static str {
        match self {
            Self::Hunting => "assets/ui/journal/hunting badge.png",
            Self::Fishing => "assets/ui/journal/fishing badge.png",
            Self::Horseback => "assets/ui/journal/horseback badge.png",
            Self::Husbandry => "assets/ui/journal/husbandry badge.png",
            Self::Mining => "assets/ui/journal/mining badge.png",
            Self::Foraging => "assets/ui/journal/foraging badge.png",
            Self::Woodcutting => "assets/ui/journal/woodcutting badge.png",
        }
    }

    pub fn fallback_color(&self) -> Color32 {
        match self {
            Self::Hunting => Color32::from_rgb(139, 69, 19),    // Saddle brown
            Self::Fishing => Color32::from_rgb(70, 130, 180),   // Steel blue
            Self::Horseback => Color32::from_rgb(160, 82, 45),  // Sienna
            Self::Husbandry => Color32::from_rgb(184, 134, 11), // Dark goldenrod
            Self::Mining => Color32::from_rgb(105, 105, 105),   // Dim gray
            Self::Foraging => Color32::from_rgb(34, 139, 34),   // Forest green
            Self::Woodcutting => Color32::from_rgb(139, 90, 43), // Brown
        }
    }

    /// Inner tabs (branches) for this perk category
    pub fn inner_tabs(&self) -> &'static [&'static str] {
        match self {
            Self::Hunting => &["Tracking", "Trapping", "Harvesting", "Companions"],
            Self::Fishing => &["Techniques", "Locations", "Tackle", "Recipes"],
            Self::Horseback => &["Bond", "Endurance", "Speed", "Combat", "Utility"],
            Self::Husbandry => &["Hounds", "Fowl", "Pens", "Breeding"],
            Self::Mining => &["Prospecting", "Extraction", "Refining", "Gems"],
            Self::Foraging => &["Berries", "Herbs", "Mushrooms", "Roots"],
            Self::Woodcutting => &["Felling", "Processing", "Hardwoods", "Softwoods"],
        }
    }

    pub fn all() -> &'static [PerkTree] {
        &[Self::Hunting, Self::Fishing, Self::Horseback, Self::Husbandry, Self::Mining, Self::Foraging, Self::Woodcutting]
    }
}

// ============================================================================
// ENCYCLOPEDIA CATEGORIES (Left tabs when in Encyclopedia section)
// ============================================================================

/// Encyclopedia categories - displayed as emblem tabs on the left
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EncyclopediaCategory {
    #[default]
    Fauna,
    Flora,
    Locations,
    Factions,
    Lore,
}

impl EncyclopediaCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fauna => "Fauna",
            Self::Flora => "Flora",
            Self::Locations => "Locations",
            Self::Factions => "Factions",
            Self::Lore => "Lore",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Fauna => "Creatures of the New World",
            Self::Flora => "Plants and herbs discovered",
            Self::Locations => "Places of interest",
            Self::Factions => "Groups and settlements",
            Self::Lore => "History and legends",
        }
    }

    pub fn emblem_path(&self) -> &'static str {
        match self {
            Self::Fauna => "assets/ui/journal/emblems/fauna.jpg",
            Self::Flora => "assets/ui/journal/emblems/flora.jpg",
            Self::Locations => "assets/ui/journal/emblems/locations.jpg",
            Self::Factions => "assets/ui/journal/emblems/factions.jpg",
            Self::Lore => "assets/ui/journal/emblems/lore.jpg",
        }
    }

    pub fn fallback_color(&self) -> Color32 {
        match self {
            Self::Fauna => Color32::from_rgb(160, 82, 45),    // Sienna
            Self::Flora => Color32::from_rgb(34, 139, 34),    // Forest green
            Self::Locations => Color32::from_rgb(70, 130, 180), // Steel blue
            Self::Factions => Color32::from_rgb(148, 103, 189), // Purple
            Self::Lore => Color32::from_rgb(218, 165, 32),    // Goldenrod
        }
    }

    pub fn all() -> &'static [EncyclopediaCategory] {
        &[Self::Fauna, Self::Flora, Self::Locations, Self::Factions, Self::Lore]
    }
}

// ============================================================================
// STAT CATEGORIES (Left tabs when in Stats section)
// ============================================================================

/// Stat categories - Commendations (achievements) and Totals (counters)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StatCategory {
    #[default]
    Commendations,
    Totals,
}

impl StatCategory {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Commendations => "Commendations",
            Self::Totals => "Totals",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Commendations => "Achievements earned (Bronze/Silver/Gold)",
            Self::Totals => "Lifetime statistics and counters",
        }
    }

    pub fn emblem_path(&self) -> &'static str {
        match self {
            Self::Commendations => "assets/ui/journal/emblems/commendations.png",
            Self::Totals => "assets/ui/journal/emblems/totals.png",
        }
    }

    pub fn fallback_color(&self) -> Color32 {
        match self {
            Self::Commendations => Color32::from_rgb(255, 215, 0),  // Gold
            Self::Totals => Color32::from_rgb(100, 149, 237),       // Cornflower blue
        }
    }

    pub fn all() -> &'static [StatCategory] {
        &[Self::Commendations, Self::Totals]
    }
}

// ============================================================================
// COMMENDATION LEVELS
// ============================================================================

/// Commendation achievement levels (3 tiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommendationLevel {
    Bronze,
    Silver,
    Gold,
}

impl CommendationLevel {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Bronze => "Bronze",
            Self::Silver => "Silver",
            Self::Gold => "Gold",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Bronze => Color32::from_rgb(205, 127, 50),
            Self::Silver => Color32::from_rgb(192, 192, 192),
            Self::Gold => Color32::from_rgb(255, 215, 0),
        }
    }
}

// ============================================================================
// JOURNAL STATE
// ============================================================================

/// State for the journal UI
#[derive(Debug, Clone)]
pub struct PerksJournalState {
    /// Current main section (sidebar selection)
    pub active_section: JournalSection,
    /// Current perk tree (when in Perks section)
    pub active_perk_tree: PerkTree,
    /// Current encyclopedia category (when in Encyclopedia section)
    pub active_encyclopedia: EncyclopediaCategory,
    /// Current stat category (when in Stats section)
    pub active_stat_category: StatCategory,
    /// Inner tab index (within current category)
    pub active_inner_tab: usize,
    /// Selected item for detail view (section, category_index, inner_tab, item_index)
    pub selected_item: Option<(JournalSection, usize, usize, usize)>,
    /// Whether the journal is open
    pub is_open: bool,
    /// Animation progress for page transitions (0.0 to 1.0)
    pub transition_progress: f32,
    /// Previous section (for transition animation)
    pub previous_section: Option<JournalSection>,
    /// Sidebar hover state for smooth highlights
    pub sidebar_hover_index: Option<usize>,
}

impl Default for PerksJournalState {
    fn default() -> Self {
        Self {
            active_section: JournalSection::Perks,
            active_perk_tree: PerkTree::Hunting,
            active_encyclopedia: EncyclopediaCategory::Fauna,
            active_stat_category: StatCategory::Commendations,
            active_inner_tab: 0,
            selected_item: None,
            is_open: false,
            transition_progress: 1.0,
            previous_section: None,
            sidebar_hover_index: None,
        }
    }
}

// ============================================================================
// JOURNAL TEXTURES
// ============================================================================

/// Loaded textures for the journal
pub struct JournalTextures {
    /// Naturalist badge - main emblem for Perks section
    pub naturalist_badge: Option<TextureHandle>,
    /// Perk tree emblem textures
    pub perk_emblems: HashMap<PerkTree, TextureHandle>,
    /// Encyclopedia category emblem textures
    pub encyclopedia_emblems: HashMap<EncyclopediaCategory, TextureHandle>,
    /// Stat category emblem textures
    pub stat_emblems: HashMap<StatCategory, TextureHandle>,
    /// Perk artwork: (tree, inner_tab, perk_index) -> texture
    pub perk_artwork: HashMap<(PerkTree, usize, usize), TextureHandle>,
    /// Main section icons
    pub section_icons: HashMap<JournalSection, TextureHandle>,
}

impl Default for JournalTextures {
    fn default() -> Self {
        Self {
            naturalist_badge: None,
            perk_emblems: HashMap::new(),
            encyclopedia_emblems: HashMap::new(),
            stat_emblems: HashMap::new(),
            perk_artwork: HashMap::new(),
            section_icons: HashMap::new(),
        }
    }
}

impl JournalTextures {
    pub fn new() -> Self {
        Self::default()
    }

    /// Path to the naturalist badge image
    pub fn naturalist_badge_path() -> &'static str {
        "assets/ui/journal/naturalist badge.png"
    }

    /// Load the naturalist badge texture if not already loaded
    pub fn load_naturalist_badge(&mut self, ctx: &egui::Context) {
        if self.naturalist_badge.is_some() {
            return;
        }

        let path = Self::naturalist_badge_path();
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(image) = image::load_from_memory(&bytes) {
                let size = [image.width() as usize, image.height() as usize];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ctx.load_texture("naturalist_badge", color_image, egui::TextureOptions::LINEAR);
                self.naturalist_badge = Some(texture);
                println!("[JOURNAL] Loaded naturalist badge texture");
            }
        }
    }

    /// Load perk category emblems from files
    pub fn load_perk_emblems(&mut self, ctx: &egui::Context) {
        for tree in PerkTree::all() {
            if self.perk_emblems.contains_key(tree) {
                continue;
            }

            let path = tree.emblem_path();
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(image) = image::load_from_memory(&bytes) {
                    let size = [image.width() as usize, image.height() as usize];
                    let image_buffer = image.to_rgba8();
                    let pixels = image_buffer.as_flat_samples();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                    let texture = ctx.load_texture(format!("perk_emblem_{:?}", tree), color_image, egui::TextureOptions::LINEAR);
                    self.perk_emblems.insert(*tree, texture);
                    println!("[JOURNAL] Loaded {} emblem", tree.name());
                }
            }
        }
    }

    pub fn get_naturalist_badge(&self) -> Option<TextureId> {
        self.naturalist_badge.as_ref().map(|h| h.id())
    }

    pub fn get_perk_emblem(&self, tree: PerkTree) -> Option<TextureId> {
        self.perk_emblems.get(&tree).map(|h| h.id())
    }

    pub fn get_encyclopedia_emblem(&self, cat: EncyclopediaCategory) -> Option<TextureId> {
        self.encyclopedia_emblems.get(&cat).map(|h| h.id())
    }

    pub fn get_stat_emblem(&self, cat: StatCategory) -> Option<TextureId> {
        self.stat_emblems.get(&cat).map(|h| h.id())
    }

    pub fn get_perk_artwork(&self, tree: PerkTree, inner_tab: usize, perk_idx: usize) -> Option<TextureId> {
        self.perk_artwork.get(&(tree, inner_tab, perk_idx)).map(|h| h.id())
    }
}

// ============================================================================
// COLORS
// ============================================================================

pub struct JournalColors {
    pub paper: Color32,
    pub leather: Color32,
    pub ink: Color32,
    pub accent: Color32,
    pub tab_active: Color32,
    pub tab_inactive: Color32,
    pub tab_hover: Color32,
    pub overlay: Color32,
    pub section_active: Color32,
    pub section_inactive: Color32,
}

impl Default for JournalColors {
    fn default() -> Self {
        Self {
            paper: Color32::from_rgb(244, 228, 188),
            leather: Color32::from_rgb(101, 67, 33),
            ink: Color32::from_rgb(40, 30, 20),
            accent: Color32::from_rgb(139, 90, 43),
            tab_active: Color32::from_rgb(210, 180, 140),
            tab_inactive: Color32::from_rgb(160, 130, 100),
            tab_hover: Color32::from_rgb(190, 160, 120),
            overlay: Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            section_active: Color32::from_rgb(180, 150, 100),
            section_inactive: Color32::from_rgb(130, 100, 70),
        }
    }
}

// ============================================================================
// EASING FUNCTIONS
// ============================================================================

/// Smooth ease-out cubic for animations
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Smooth ease-in-out for hover effects
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// ============================================================================
// MAIN RENDER FUNCTION
// ============================================================================

pub fn render_perks_journal(
    ui_ctx: &egui::Context,
    state: &mut PerksJournalState,
    textures: &mut JournalTextures,
) {
    // Load textures on first render
    textures.load_naturalist_badge(ui_ctx);
    textures.load_perk_emblems(ui_ctx);

    let colors = JournalColors::default();

    let screen_rect = ui_ctx.screen_rect();
    let screen_width = screen_rect.width();
    let screen_height = screen_rect.height();

    // Update transition animation
    if state.transition_progress < 1.0 {
        state.transition_progress = (state.transition_progress + 0.06).min(1.0);
        ui_ctx.request_repaint();
    }

    // Layout: centered journal with left tabs, right content
    let journal_width = (screen_width * 0.75).min(1000.0);
    let journal_height = (screen_height * 0.80).min(700.0);
    // Use screen_rect min position for proper centering
    let journal_left = screen_rect.min.x + (screen_width - journal_width) / 2.0;
    let journal_top = screen_rect.min.y + (screen_height - journal_height) / 2.0;

    // Tab dimensions (left side book tabs)
    let tab_width = 140.0;
    let content_width = journal_width - tab_width;

    // Semi-transparent overlay with click-to-close
    egui::Area::new(egui::Id::new("journal_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Middle)
        .interactable(true)
        .show(ui_ctx, |ui| {
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(screen_width, screen_height));
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 140));

            // Click outside journal to close
            let response = ui.allocate_rect(rect, egui::Sense::click());
            if response.clicked() {
                // Only close if click is outside the journal bounds
                if let Some(pos) = response.interact_pointer_pos() {
                    let journal_rect = egui::Rect::from_min_size(
                        egui::pos2(journal_left, journal_top),
                        egui::vec2(journal_width, journal_height),
                    );
                    if !journal_rect.contains(pos) {
                        state.is_open = false;
                    }
                }
            }
        });

    // === MAIN JOURNAL PANEL ===
    egui::Area::new(egui::Id::new("journal_main"))
        .fixed_pos(egui::pos2(journal_left, journal_top))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ui_ctx, |ui| {
            // Consume clicks within journal area
            let journal_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(journal_width, journal_height));
            ui.allocate_rect(journal_rect, egui::Sense::click());

            // === LEFT SIDE: NAVIGATION TABS ===
            let tab_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(tab_width, journal_height));

            // Dark leather background for tabs
            ui.painter().rect_filled(
                tab_rect,
                egui::Rounding { nw: 12.0, sw: 12.0, ne: 0.0, se: 0.0 },
                Color32::from_rgb(50, 40, 30),
            );

            // Navigation items
            let nav_items = [
                (JournalSection::Perks, "Journal", "📖"),
                (JournalSection::Stats, "Stats", "📊"),
                (JournalSection::Encyclopedia, "Encyclopedia", "🔍"),
            ];

            let item_height = 50.0;
            let mut y = 20.0;

            for (section, label, icon) in nav_items.iter() {
                let item_rect = egui::Rect::from_min_size(
                    egui::pos2(8.0, y),
                    egui::vec2(tab_width - 16.0, item_height),
                );

                let is_active = state.active_section == *section;
                let response = ui.allocate_rect(item_rect, egui::Sense::click());

                let hover_t = ui_ctx.animate_bool(
                    egui::Id::new(format!("nav_{:?}", section)),
                    response.hovered() || is_active,
                );

                // Background with hover/active effect
                let bg_alpha = (hover_t * 180.0) as u8;
                let bg_color = if is_active {
                    Color32::from_rgb(139, 90, 43)
                } else {
                    Color32::from_rgba_unmultiplied(100, 80, 60, bg_alpha)
                };
                ui.painter().rect_filled(item_rect, egui::Rounding::same(6.0), bg_color);

                // Active indicator
                if is_active {
                    let indicator = egui::Rect::from_min_size(
                        egui::pos2(0.0, y + 12.0),
                        egui::vec2(4.0, item_height - 24.0),
                    );
                    ui.painter().rect_filled(indicator, egui::Rounding::same(2.0), Color32::from_rgb(255, 200, 100));
                }

                // Icon and label
                let text_color = if is_active { Color32::WHITE } else { Color32::from_rgb(200, 180, 160) };
                ui.painter().text(
                    egui::pos2(item_rect.min.x + 15.0, item_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *icon,
                    egui::FontId::proportional(20.0),
                    text_color,
                );
                ui.painter().text(
                    egui::pos2(item_rect.min.x + 45.0, item_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *label,
                    egui::FontId::proportional(14.0),
                    text_color,
                );

                if response.clicked() && !is_active {
                    state.previous_section = Some(state.active_section);
                    state.active_section = *section;
                    state.transition_progress = 0.0;
                    state.active_inner_tab = 0;
                    state.selected_item = None;
                }

                y += item_height + 6.0;
            }

            // Separator
            y += 10.0;
            ui.painter().line_segment(
                [egui::pos2(20.0, y), egui::pos2(tab_width - 20.0, y)],
                egui::Stroke::new(1.0, Color32::from_rgb(80, 65, 50)),
            );
            y += 15.0;

            // Settings at bottom
            let settings_y = journal_height - item_height - 20.0;
            let settings_rect = egui::Rect::from_min_size(
                egui::pos2(8.0, settings_y),
                egui::vec2(tab_width - 16.0, item_height),
            );

            let is_settings_active = state.active_section == JournalSection::Settings;
            let settings_response = ui.allocate_rect(settings_rect, egui::Sense::click());

            let settings_hover_t = ui_ctx.animate_bool(
                egui::Id::new("nav_settings"),
                settings_response.hovered() || is_settings_active,
            );

            let settings_bg = if is_settings_active {
                Color32::from_rgb(139, 90, 43)
            } else {
                Color32::from_rgba_unmultiplied(100, 80, 60, (settings_hover_t * 180.0) as u8)
            };
            ui.painter().rect_filled(settings_rect, egui::Rounding::same(6.0), settings_bg);

            if is_settings_active {
                let indicator = egui::Rect::from_min_size(
                    egui::pos2(0.0, settings_y + 12.0),
                    egui::vec2(4.0, item_height - 24.0),
                );
                ui.painter().rect_filled(indicator, egui::Rounding::same(2.0), Color32::from_rgb(255, 200, 100));
            }

            let settings_text_color = if is_settings_active { Color32::WHITE } else { Color32::from_rgb(200, 180, 160) };
            ui.painter().text(
                egui::pos2(settings_rect.min.x + 15.0, settings_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "⚙",
                egui::FontId::proportional(20.0),
                settings_text_color,
            );
            ui.painter().text(
                egui::pos2(settings_rect.min.x + 45.0, settings_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Settings",
                egui::FontId::proportional(14.0),
                settings_text_color,
            );

            if settings_response.clicked() && !is_settings_active {
                state.previous_section = Some(state.active_section);
                state.active_section = JournalSection::Settings;
                state.transition_progress = 0.0;
            }

            // === RIGHT SIDE: CONTENT AREA (Book Pages) ===
            let content_rect = egui::Rect::from_min_size(
                egui::pos2(tab_width, 0.0),
                egui::vec2(content_width, journal_height),
            );

            // Paper background
            ui.painter().rect_filled(
                content_rect,
                egui::Rounding { nw: 0.0, sw: 0.0, ne: 12.0, se: 12.0 },
                colors.paper,
            );

            // Leather border
            ui.painter().rect_stroke(
                content_rect,
                egui::Rounding { nw: 0.0, sw: 0.0, ne: 12.0, se: 12.0 },
                egui::Stroke::new(3.0, colors.leather),
            );

            // Page transition effect
            let transition_t = ease_out_cubic(state.transition_progress);
            let content_alpha = (transition_t * 255.0) as u8;
            let slide_offset = (1.0 - transition_t) * 15.0;

            // Content area margins
            let margin = 20.0;
            let header_y = margin + slide_offset;

            // Section title
            let section_title = match state.active_section {
                JournalSection::Perks => "NATURALIST JOURNAL",
                JournalSection::Stats => "STATISTICS",
                JournalSection::Encyclopedia => "ENCYCLOPEDIA",
                JournalSection::Settings => "SETTINGS",
            };

            ui.painter().text(
                egui::pos2(tab_width + content_width / 2.0, header_y + 15.0),
                egui::Align2::CENTER_CENTER,
                section_title,
                egui::FontId::proportional(22.0),
                Color32::from_rgba_unmultiplied(colors.ink.r(), colors.ink.g(), colors.ink.b(), content_alpha),
            );

            // Decorative underline
            ui.painter().line_segment(
                [
                    egui::pos2(tab_width + content_width * 0.2, header_y + 35.0),
                    egui::pos2(tab_width + content_width * 0.8, header_y + 35.0),
                ],
                egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(colors.leather.r(), colors.leather.g(), colors.leather.b(), content_alpha)),
            );

            // Subsection tabs for applicable sections
            let mut content_top = header_y + 50.0;

            match state.active_section {
                JournalSection::Perks => {
                    let tabs = PerkTree::all();
                    content_top = render_content_tabs(ui, ui_ctx, tabs.iter().map(|t| t.name()),
                        tabs.iter().position(|t| *t == state.active_perk_tree).unwrap_or(0),
                        content_top, tab_width, content_width, &colors, content_alpha,
                        |i| { state.active_perk_tree = tabs[i]; state.selected_item = None; });
                }
                JournalSection::Stats => {
                    let tabs = StatCategory::all();
                    content_top = render_content_tabs(ui, ui_ctx, tabs.iter().map(|t| t.name()),
                        tabs.iter().position(|t| *t == state.active_stat_category).unwrap_or(0),
                        content_top, tab_width, content_width, &colors, content_alpha,
                        |i| { state.active_stat_category = tabs[i]; });
                }
                JournalSection::Encyclopedia => {
                    let tabs = EncyclopediaCategory::all();
                    content_top = render_content_tabs(ui, ui_ctx, tabs.iter().map(|t| t.name()),
                        tabs.iter().position(|t| *t == state.active_encyclopedia).unwrap_or(0),
                        content_top, tab_width, content_width, &colors, content_alpha,
                        |i| { state.active_encyclopedia = tabs[i]; });
                }
                JournalSection::Settings => {}
            }

            // Main content
            let content_height = journal_height - content_top - margin;
            let half_width = (content_width - margin * 3.0) / 2.0;
            let left_x = tab_width + margin;
            let right_x = tab_width + margin * 2.0 + half_width;

            let left_rect = egui::Rect::from_min_size(egui::pos2(left_x, content_top), egui::vec2(half_width, content_height));
            let right_rect = egui::Rect::from_min_size(egui::pos2(right_x, content_top), egui::vec2(half_width, content_height));

            // Center spine line
            let spine_x = tab_width + content_width / 2.0;
            ui.painter().line_segment(
                [egui::pos2(spine_x, content_top - 10.0), egui::pos2(spine_x, journal_height - margin)],
                egui::Stroke::new(2.0, Color32::from_rgba_unmultiplied(colors.leather.r(), colors.leather.g(), colors.leather.b(), 100)),
            );

            match state.active_section {
                JournalSection::Perks => {
                    render_perk_list(ui, left_rect, state, &colors);
                    render_perk_detail(ui, right_rect, state, textures, &colors);
                }
                JournalSection::Stats => {
                    render_stats_content(ui, left_rect, right_rect, state, &colors);
                }
                JournalSection::Encyclopedia => {
                    render_encyclopedia_content(ui, left_rect, right_rect, state, &colors);
                }
                JournalSection::Settings => {
                    // Rendered in main.rs with SharedState access
                }
            }

            // Close button
            let close_rect = egui::Rect::from_min_size(
                egui::pos2(journal_width - 35.0, 10.0),
                egui::vec2(25.0, 25.0),
            );
            let close_response = ui.allocate_rect(close_rect, egui::Sense::click());
            let close_hover = ui_ctx.animate_bool(egui::Id::new("close_btn"), close_response.hovered());

            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                egui::FontId::proportional(16.0),
                Color32::from_rgba_unmultiplied(80, 60, 40, (150.0 + close_hover * 105.0) as u8),
            );

            if close_response.clicked() {
                state.is_open = false;
            }
        });
}

/// Render horizontal content tabs with click handling
fn render_content_tabs<'a, F>(
    ui: &mut egui::Ui,
    ui_ctx: &egui::Context,
    tabs: impl Iterator<Item = &'a str>,
    active_index: usize,
    y_pos: f32,
    left_offset: f32,
    content_width: f32,
    colors: &JournalColors,
    alpha: u8,
    mut on_click: F,
) -> f32
where
    F: FnMut(usize),
{
    let tabs: Vec<&str> = tabs.collect();
    let tab_height = 26.0;
    let tab_spacing = 8.0;
    let tab_width = 75.0;
    let total_width = (tabs.len() as f32) * tab_width + ((tabs.len() - 1) as f32) * tab_spacing;
    let start_x = left_offset + (content_width - total_width) / 2.0;

    for (i, tab_name) in tabs.iter().enumerate() {
        let tab_x = start_x + (i as f32) * (tab_width + tab_spacing);
        let tab_rect = egui::Rect::from_min_size(
            egui::pos2(tab_x, y_pos),
            egui::vec2(tab_width, tab_height),
        );

        let is_active = i == active_index;
        let response = ui.allocate_rect(tab_rect, egui::Sense::click());
        let hover_t = ui_ctx.animate_bool(egui::Id::new(format!("ctab_{}_{}", y_pos as i32, i)), response.hovered());

        let tab_color = if is_active {
            Color32::from_rgba_unmultiplied(colors.tab_active.r(), colors.tab_active.g(), colors.tab_active.b(), alpha)
        } else {
            let base = colors.tab_inactive;
            let hover = colors.tab_hover;
            Color32::from_rgba_unmultiplied(
                (base.r() as f32 + (hover.r() as f32 - base.r() as f32) * hover_t) as u8,
                (base.g() as f32 + (hover.g() as f32 - base.g() as f32) * hover_t) as u8,
                (base.b() as f32 + (hover.b() as f32 - base.b() as f32) * hover_t) as u8,
                alpha,
            )
        };

        ui.painter().rect_filled(tab_rect, egui::Rounding::same(4.0), tab_color);

        if is_active {
            let underline = egui::Rect::from_min_size(
                egui::pos2(tab_x + 8.0, y_pos + tab_height - 3.0),
                egui::vec2(tab_width - 16.0, 2.0),
            );
            ui.painter().rect_filled(underline, egui::Rounding::same(1.0),
                Color32::from_rgba_unmultiplied(colors.accent.r(), colors.accent.g(), colors.accent.b(), alpha));
        }

        ui.painter().text(
            tab_rect.center(),
            egui::Align2::CENTER_CENTER,
            *tab_name,
            egui::FontId::proportional(11.0),
            Color32::from_rgba_unmultiplied(colors.ink.r(), colors.ink.g(), colors.ink.b(), alpha),
        );

        if response.clicked() {
            on_click(i);
        }
    }

    y_pos + tab_height + 12.0
}

// ============================================================================
// EMBLEM TAB RENDERER (Returns clicked index) - Legacy, kept for compatibility
// ============================================================================

fn render_emblem_tabs_perk(
    ui: &mut egui::Ui,
    items: &[PerkTree],
    active: PerkTree,
    textures: &JournalTextures,
    width: f32,
    height: f32,
    spacing: f32,
    colors: &JournalColors,
) -> Option<PerkTree> {
    let mut clicked = None;
    for (i, item) in items.iter().enumerate() {
        let tab_y = (i as f32) * (height + spacing);
        let tab_rect = egui::Rect::from_min_size(egui::pos2(0.0, tab_y), egui::vec2(width, height));

        let is_active = active == *item;
        let response = ui.allocate_rect(tab_rect, egui::Sense::click());

        let bg_color = if is_active { colors.tab_active } else if response.hovered() { colors.tab_hover } else { colors.tab_inactive };

        if let Some(tex_id) = textures.get_perk_emblem(*item) {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            ui.painter().image(tex_id, tab_rect.shrink(5.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        } else {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            let inner = tab_rect.shrink(6.0);
            ui.painter().rect_filled(inner, egui::Rounding::same(4.0), item.fallback_color());
            ui.painter().text(inner.center(), egui::Align2::CENTER_CENTER, item.name().chars().next().unwrap_or('?').to_string(), egui::FontId::proportional(24.0), Color32::WHITE);
        }

        if is_active { ui.painter().rect_stroke(tab_rect, egui::Rounding::same(8.0), egui::Stroke::new(3.0, colors.accent)); }
        if response.clicked() { clicked = Some(*item); }
        if response.hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new(format!("perk_emblem_{}", i)), |ui| {
                ui.label(item.name());
                ui.label(egui::RichText::new(item.description()).weak());
            });
        }
    }
    clicked
}

fn render_emblem_tabs_stat(
    ui: &mut egui::Ui,
    items: &[StatCategory],
    active: StatCategory,
    textures: &JournalTextures,
    width: f32,
    height: f32,
    spacing: f32,
    colors: &JournalColors,
) -> Option<StatCategory> {
    let mut clicked = None;
    for (i, item) in items.iter().enumerate() {
        let tab_y = (i as f32) * (height + spacing);
        let tab_rect = egui::Rect::from_min_size(egui::pos2(0.0, tab_y), egui::vec2(width, height));

        let is_active = active == *item;
        let response = ui.allocate_rect(tab_rect, egui::Sense::click());

        let bg_color = if is_active { colors.tab_active } else if response.hovered() { colors.tab_hover } else { colors.tab_inactive };

        if let Some(tex_id) = textures.get_stat_emblem(*item) {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            ui.painter().image(tex_id, tab_rect.shrink(5.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        } else {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            let inner = tab_rect.shrink(6.0);
            ui.painter().rect_filled(inner, egui::Rounding::same(4.0), item.fallback_color());
            ui.painter().text(inner.center(), egui::Align2::CENTER_CENTER, item.name().chars().next().unwrap_or('?').to_string(), egui::FontId::proportional(24.0), Color32::WHITE);
        }

        if is_active { ui.painter().rect_stroke(tab_rect, egui::Rounding::same(8.0), egui::Stroke::new(3.0, colors.accent)); }
        if response.clicked() { clicked = Some(*item); }
        if response.hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new(format!("stat_emblem_{}", i)), |ui| {
                ui.label(item.name());
                ui.label(egui::RichText::new(item.description()).weak());
            });
        }
    }
    clicked
}

fn render_emblem_tabs_encyclopedia(
    ui: &mut egui::Ui,
    items: &[EncyclopediaCategory],
    active: EncyclopediaCategory,
    textures: &JournalTextures,
    width: f32,
    height: f32,
    spacing: f32,
    colors: &JournalColors,
) -> Option<EncyclopediaCategory> {
    let mut clicked = None;
    for (i, item) in items.iter().enumerate() {
        let tab_y = (i as f32) * (height + spacing);
        let tab_rect = egui::Rect::from_min_size(egui::pos2(0.0, tab_y), egui::vec2(width, height));

        let is_active = active == *item;
        let response = ui.allocate_rect(tab_rect, egui::Sense::click());

        let bg_color = if is_active { colors.tab_active } else if response.hovered() { colors.tab_hover } else { colors.tab_inactive };

        if let Some(tex_id) = textures.get_encyclopedia_emblem(*item) {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            ui.painter().image(tex_id, tab_rect.shrink(5.0), egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        } else {
            ui.painter().rect_filled(tab_rect, egui::Rounding::same(8.0), bg_color);
            let inner = tab_rect.shrink(6.0);
            ui.painter().rect_filled(inner, egui::Rounding::same(4.0), item.fallback_color());
            ui.painter().text(inner.center(), egui::Align2::CENTER_CENTER, item.name().chars().next().unwrap_or('?').to_string(), egui::FontId::proportional(24.0), Color32::WHITE);
        }

        if is_active { ui.painter().rect_stroke(tab_rect, egui::Rounding::same(8.0), egui::Stroke::new(3.0, colors.accent)); }
        if response.clicked() { clicked = Some(*item); }
        if response.hovered() {
            egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new(format!("enc_emblem_{}", i)), |ui| {
                ui.label(item.name());
                ui.label(egui::RichText::new(item.description()).weak());
            });
        }
    }
    clicked
}

// ============================================================================
// PERK LIST & DETAIL
// ============================================================================

fn render_perk_list(ui: &mut egui::Ui, rect: egui::Rect, state: &mut PerksJournalState, colors: &JournalColors) {
    ui.painter().text(
        egui::pos2(rect.center().x, rect.min.y + 15.0),
        egui::Align2::CENTER_CENTER,
        "PERKS",
        egui::FontId::proportional(16.0),
        colors.ink,
    );

    // Placeholder entries
    let perks = [
        ("Tier I Perk", "Basic ability", true),
        ("Tier I Perk B", "Another starter", true),
        ("Tier II Perk", "Intermediate", false),
        ("Tier III Perk", "Advanced", false),
        ("Tier IV Perk", "Expert", false),
        ("Tier V Perk", "Legendary", false),
    ];

    let mut y = rect.min.y + 40.0;
    let entry_height = 45.0;

    for (i, (name, desc, unlocked)) in perks.iter().enumerate() {
        let entry_rect = egui::Rect::from_min_size(
            egui::pos2(rect.min.x + 8.0, y),
            egui::vec2(rect.width() - 16.0, entry_height - 4.0),
        );

        let response = ui.allocate_rect(entry_rect, egui::Sense::click());
        let tree_idx = PerkTree::all().iter().position(|t| *t == state.active_perk_tree).unwrap_or(0);
        let is_selected = state.selected_item == Some((JournalSection::Perks, tree_idx, state.active_inner_tab, i));

        let bg = if is_selected { colors.tab_active } else if response.hovered() { colors.tab_hover } else { Color32::TRANSPARENT };
        if bg != Color32::TRANSPARENT {
            ui.painter().rect_filled(entry_rect, egui::Rounding::same(4.0), bg);
        }

        let icon = if *unlocked { "[+]" } else { "[ ]" };
        let text_color = if *unlocked { colors.ink } else { Color32::DARK_GRAY };

        ui.painter().text(egui::pos2(entry_rect.min.x + 5.0, entry_rect.min.y + 10.0), egui::Align2::LEFT_CENTER, icon, egui::FontId::monospace(11.0), text_color);
        ui.painter().text(egui::pos2(entry_rect.min.x + 30.0, entry_rect.min.y + 10.0), egui::Align2::LEFT_CENTER, *name, egui::FontId::proportional(13.0), text_color);
        ui.painter().text(egui::pos2(entry_rect.min.x + 30.0, entry_rect.min.y + 26.0), egui::Align2::LEFT_CENTER, *desc, egui::FontId::proportional(10.0), Color32::DARK_GRAY);

        if response.clicked() {
            state.selected_item = Some((JournalSection::Perks, tree_idx, state.active_inner_tab, i));
        }

        y += entry_height;
    }
}

fn render_perk_detail(ui: &mut egui::Ui, rect: egui::Rect, state: &PerksJournalState, textures: &JournalTextures, colors: &JournalColors) {
    if let Some((JournalSection::Perks, _tree_idx, inner_tab, perk_idx)) = state.selected_item {
        ui.painter().text(egui::pos2(rect.center().x, rect.min.y + 15.0), egui::Align2::CENTER_CENTER, "DETAILS", egui::FontId::proportional(16.0), colors.ink);

        let artwork_size = 180.0;
        let artwork_rect = egui::Rect::from_min_size(
            egui::pos2(rect.center().x - artwork_size / 2.0, rect.min.y + 40.0),
            egui::vec2(artwork_size, artwork_size),
        );

        if let Some(tex_id) = textures.get_perk_artwork(state.active_perk_tree, inner_tab, perk_idx) {
            ui.painter().image(tex_id, artwork_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        } else {
            ui.painter().rect_stroke(artwork_rect, egui::Rounding::same(8.0), egui::Stroke::new(2.0, colors.accent));
            ui.painter().text(artwork_rect.center(), egui::Align2::CENTER_CENTER, "Artwork\nSlot", egui::FontId::proportional(14.0), Color32::DARK_GRAY);
        }

        let text_y = artwork_rect.max.y + 20.0;
        ui.painter().text(egui::pos2(rect.center().x, text_y), egui::Align2::CENTER_CENTER, "Perk Name", egui::FontId::proportional(16.0), colors.ink);
        ui.painter().text(egui::pos2(rect.center().x, text_y + 22.0), egui::Align2::CENTER_CENTER, "Description here", egui::FontId::proportional(12.0), colors.ink);

        // Unlock button
        let btn_rect = egui::Rect::from_min_size(egui::pos2(rect.center().x - 55.0, rect.max.y - 45.0), egui::vec2(110.0, 32.0));
        let btn_response = ui.allocate_rect(btn_rect, egui::Sense::click());
        let btn_color = if btn_response.hovered() { colors.tab_hover } else { colors.tab_inactive };
        ui.painter().rect_filled(btn_rect, egui::Rounding::same(6.0), btn_color);
        ui.painter().rect_stroke(btn_rect, egui::Rounding::same(6.0), egui::Stroke::new(2.0, colors.leather));
        ui.painter().text(btn_rect.center(), egui::Align2::CENTER_CENTER, "UNLOCK", egui::FontId::proportional(13.0), colors.ink);
    } else {
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Select a perk", egui::FontId::proportional(14.0), Color32::DARK_GRAY);
    }
}

// ============================================================================
// STATS CONTENT
// ============================================================================

fn render_stats_content(ui: &mut egui::Ui, left_rect: egui::Rect, right_rect: egui::Rect, state: &PerksJournalState, colors: &JournalColors) {
    match state.active_stat_category {
        StatCategory::Commendations => {
            // Left page: Commendations list
            ui.painter().text(egui::pos2(left_rect.center().x, left_rect.min.y + 15.0), egui::Align2::CENTER_CENTER, "COMMENDATIONS", egui::FontId::proportional(16.0), colors.ink);
            ui.painter().text(egui::pos2(left_rect.min.x + 20.0, left_rect.min.y + 45.0), egui::Align2::LEFT_CENTER, "Bronze → Silver → Gold", egui::FontId::proportional(11.0), Color32::DARK_GRAY);

            let commendations = [
                ("First Blood", CommendationLevel::Gold),
                ("Big Game Hunter", CommendationLevel::Silver),
                ("Master Angler", CommendationLevel::Bronze),
                ("Trail Blazer", CommendationLevel::Silver),
                ("Horse Whisperer", CommendationLevel::Bronze),
                ("Forager", CommendationLevel::Gold),
                ("Lumberjack", CommendationLevel::Bronze),
                ("Prospector", CommendationLevel::Bronze),
            ];
            let mut y = left_rect.min.y + 70.0;
            for (name, level) in commendations {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(egui::pos2(left_rect.min.x + 10.0, y - 6.0), egui::vec2(left_rect.width() - 20.0, 24.0)),
                    egui::Rounding::same(4.0),
                    Color32::from_rgba_unmultiplied(level.color().r(), level.color().g(), level.color().b(), 40),
                );
                ui.painter().text(egui::pos2(left_rect.min.x + 20.0, y + 5.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(12.0), colors.ink);
                ui.painter().text(egui::pos2(left_rect.max.x - 20.0, y + 5.0), egui::Align2::RIGHT_CENTER, level.name(), egui::FontId::proportional(10.0), level.color());
                y += 30.0;
            }

            // Right page: Selected commendation details
            ui.painter().text(right_rect.center(), egui::Align2::CENTER_CENTER, "Select a commendation\nto view progress", egui::FontId::proportional(14.0), Color32::DARK_GRAY);
        }
        StatCategory::Totals => {
            // Left page: Lifetime totals
            ui.painter().text(egui::pos2(left_rect.center().x, left_rect.min.y + 15.0), egui::Align2::CENTER_CENTER, "LIFETIME TOTALS", egui::FontId::proportional(16.0), colors.ink);

            let totals = [
                ("Animals Hunted", "47"),
                ("Fish Caught", "23"),
                ("Miles Ridden", "156"),
                ("Hounds Raised", "2"),
                ("Ore Mined", "340 lbs"),
                ("Plants Foraged", "89"),
                ("Trees Felled", "64"),
                ("Days Survived", "42"),
            ];
            let mut y = left_rect.min.y + 50.0;
            for (label, value) in totals {
                ui.painter().text(egui::pos2(left_rect.min.x + 20.0, y), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(12.0), colors.ink);
                ui.painter().text(egui::pos2(left_rect.max.x - 20.0, y), egui::Align2::RIGHT_CENTER, value, egui::FontId::proportional(12.0), colors.accent);
                y += 28.0;
            }

            // Right page: More totals or breakdown
            ui.painter().text(egui::pos2(right_rect.center().x, right_rect.min.y + 15.0), egui::Align2::CENTER_CENTER, "RECORDS", egui::FontId::proportional(16.0), colors.ink);

            let records = [
                ("Largest Fish", "12 lb Bass"),
                ("Biggest Game", "Black Bear"),
                ("Fastest Horse", "Midnight"),
                ("Rarest Find", "Gold Nugget"),
                ("Longest Hunt", "3 hours"),
            ];
            let mut y = right_rect.min.y + 50.0;
            for (label, value) in records {
                ui.painter().text(egui::pos2(right_rect.min.x + 20.0, y), egui::Align2::LEFT_CENTER, label, egui::FontId::proportional(12.0), colors.ink);
                ui.painter().text(egui::pos2(right_rect.max.x - 20.0, y), egui::Align2::RIGHT_CENTER, value, egui::FontId::proportional(11.0), colors.accent);
                y += 28.0;
            }
        }
    }
}

// ============================================================================
// ENCYCLOPEDIA CONTENT
// ============================================================================

/// Discovery tier for encyclopedia entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryTier {
    Unknown,    // Not yet discovered - shows as "???"
    Sighted,    // Tier 1 - Name revealed
    Observed,   // Tier 2
    Studied,    // Tier 3
    Mastered,   // Tier 4 - Full knowledge
}

impl DiscoveryTier {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Unknown => "???",
            Self::Sighted => "Sighted",
            Self::Observed => "Observed",
            Self::Studied => "Studied",
            Self::Mastered => "Mastered",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Unknown => Color32::DARK_GRAY,
            Self::Sighted => Color32::from_rgb(205, 127, 50),   // Bronze
            Self::Observed => Color32::from_rgb(192, 192, 192), // Silver
            Self::Studied => Color32::from_rgb(255, 215, 0),    // Gold
            Self::Mastered => Color32::from_rgb(50, 205, 50),   // Lime green
        }
    }

    pub fn is_discovered(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

fn render_encyclopedia_content(ui: &mut egui::Ui, left_rect: egui::Rect, right_rect: egui::Rect, state: &PerksJournalState, colors: &JournalColors) {
    ui.painter().text(egui::pos2(left_rect.center().x, left_rect.min.y + 15.0), egui::Align2::CENTER_CENTER, "ENTRIES", egui::FontId::proportional(16.0), colors.ink);

    // Entries: (real_name, tier) - name shows as "???" if Unknown
    let entries: &[(&str, DiscoveryTier)] = match state.active_encyclopedia {
        EncyclopediaCategory::Fauna => &[
            ("White-Tailed Deer", DiscoveryTier::Mastered),
            ("Wild Boar", DiscoveryTier::Studied),
            ("Black Bear", DiscoveryTier::Observed),
            ("Gray Wolf", DiscoveryTier::Sighted),
            ("Eastern Cougar", DiscoveryTier::Unknown),
            ("American Bison", DiscoveryTier::Unknown),
            ("Wild Turkey", DiscoveryTier::Unknown),
            ("Timber Rattlesnake", DiscoveryTier::Unknown),
        ],
        EncyclopediaCategory::Flora => &[
            ("Oak Tree", DiscoveryTier::Mastered),
            ("Wild Mint", DiscoveryTier::Studied),
            ("Foxglove", DiscoveryTier::Sighted),
            ("Virginia Creeper", DiscoveryTier::Unknown),
            ("Poison Ivy", DiscoveryTier::Unknown),
            ("Wild Ginseng", DiscoveryTier::Unknown),
            ("Sassafras", DiscoveryTier::Unknown),
        ],
        EncyclopediaCategory::Locations => &[
            ("Roanoke Settlement", DiscoveryTier::Mastered),
            ("Croatoan Village", DiscoveryTier::Observed),
            ("Hidden Cove", DiscoveryTier::Unknown),
            ("Ancient Burial Ground", DiscoveryTier::Unknown),
        ],
        EncyclopediaCategory::Factions => &[
            ("English Colonists", DiscoveryTier::Studied),
            ("Croatoan Tribe", DiscoveryTier::Sighted),
            ("Spanish Explorers", DiscoveryTier::Unknown),
            ("Lost Expedition", DiscoveryTier::Unknown),
        ],
        EncyclopediaCategory::Lore => &[
            ("The Disappearance", DiscoveryTier::Sighted),
            ("CROATOAN Carving", DiscoveryTier::Observed),
            ("Spirit Legends", DiscoveryTier::Unknown),
            ("Ancient Artifacts", DiscoveryTier::Unknown),
        ],
    };

    let mut y = left_rect.min.y + 45.0;
    for (real_name, tier) in entries {
        let display_name = if tier.is_discovered() { *real_name } else { "???" };
        let name_color = if tier.is_discovered() { colors.ink } else { Color32::DARK_GRAY };

        ui.painter().text(egui::pos2(left_rect.min.x + 20.0, y), egui::Align2::LEFT_CENTER, display_name, egui::FontId::proportional(12.0), name_color);
        ui.painter().text(egui::pos2(left_rect.max.x - 20.0, y), egui::Align2::RIGHT_CENTER, tier.display_name(), egui::FontId::proportional(10.0), tier.color());
        y += 28.0;
    }

    ui.painter().text(right_rect.center(), egui::Align2::CENTER_CENTER, "Select an entry", egui::FontId::proportional(14.0), Color32::DARK_GRAY);
}
