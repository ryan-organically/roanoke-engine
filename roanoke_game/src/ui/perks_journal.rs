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
}

impl JournalSection {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Perks => "Perks",
            Self::Stats => "Stats",
            Self::Encyclopedia => "Encyclopedia",
        }
    }

    pub fn all() -> &'static [JournalSection] {
        &[Self::Perks, Self::Stats, Self::Encyclopedia]
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
    /// Current main section (top tabs)
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

    let journal_width = (screen_width * 0.85).min(1200.0);
    let journal_height = (screen_height * 0.85).min(800.0);
    let journal_left = (screen_width - journal_width) / 2.0;
    let journal_top = (screen_height - journal_height) / 2.0;

    // Left emblem tabs
    let emblem_width = 70.0;
    let emblem_height = 70.0;
    let emblem_spacing = 8.0;
    let emblems_left = journal_left - emblem_width - 10.0;

    // Dark overlay
    egui::Area::new(egui::Id::new("journal_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ui_ctx, |ui| {
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(screen_width, screen_height));
            ui.painter().rect_filled(rect, 0.0, colors.overlay);
        });

    // === LEFT EMBLEM TABS ===
    let mut clicked_perk: Option<PerkTree> = None;
    let mut clicked_stat: Option<StatCategory> = None;
    let mut clicked_enc: Option<EncyclopediaCategory> = None;

    egui::Area::new(egui::Id::new("journal_emblems"))
        .fixed_pos(egui::pos2(emblems_left, journal_top + 60.0))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            match state.active_section {
                JournalSection::Perks => {
                    clicked_perk = render_emblem_tabs_perk(ui, PerkTree::all(), state.active_perk_tree, textures, emblem_width, emblem_height, emblem_spacing, &colors);
                }
                JournalSection::Stats => {
                    clicked_stat = render_emblem_tabs_stat(ui, StatCategory::all(), state.active_stat_category, textures, emblem_width, emblem_height, emblem_spacing, &colors);
                }
                JournalSection::Encyclopedia => {
                    clicked_enc = render_emblem_tabs_encyclopedia(ui, EncyclopediaCategory::all(), state.active_encyclopedia, textures, emblem_width, emblem_height, emblem_spacing, &colors);
                }
            }
        });

    // Handle emblem clicks after the area
    if let Some(tree) = clicked_perk {
        state.active_perk_tree = tree;
        state.active_inner_tab = 0;
        state.selected_item = None;
    }
    if let Some(cat) = clicked_stat {
        state.active_stat_category = cat;
        state.active_inner_tab = 0;
        state.selected_item = None;
    }
    if let Some(cat) = clicked_enc {
        state.active_encyclopedia = cat;
        state.active_inner_tab = 0;
        state.selected_item = None;
    }

    // === MAIN BOOK ===
    egui::Area::new(egui::Id::new("journal_book"))
        .fixed_pos(egui::pos2(journal_left, journal_top))
        .order(egui::Order::Foreground)
        .show(ui_ctx, |ui| {
            let book_rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(journal_width, journal_height));

            // Leather binding
            ui.painter().rect_filled(book_rect, egui::Rounding::same(12.0), colors.leather);

            // Paper interior
            let paper_margin = 15.0;
            let paper_rect = book_rect.shrink(paper_margin);
            ui.painter().rect_filled(paper_rect, egui::Rounding::same(6.0), colors.paper);

            // Center spine
            let spine_x = journal_width / 2.0;
            ui.painter().line_segment(
                [egui::pos2(spine_x, paper_margin + 10.0), egui::pos2(spine_x, journal_height - paper_margin - 10.0)],
                egui::Stroke::new(3.0, colors.leather),
            );

            // === TOP SECTION TABS (Perks / Stats / Encyclopedia) ===
            let section_tab_y = paper_margin + 12.0;
            let section_tab_width = 120.0;
            let section_tab_height = 32.0;
            let section_spacing = 15.0;
            let sections = JournalSection::all();
            let total_section_width = (sections.len() as f32) * section_tab_width + ((sections.len() - 1) as f32) * section_spacing;
            let section_start_x = (journal_width - total_section_width) / 2.0;

            for (i, section) in sections.iter().enumerate() {
                let tab_x = section_start_x + (i as f32) * (section_tab_width + section_spacing);
                let tab_rect = egui::Rect::from_min_size(
                    egui::pos2(tab_x, section_tab_y),
                    egui::vec2(section_tab_width, section_tab_height),
                );

                let is_active = state.active_section == *section;
                let response = ui.allocate_rect(tab_rect, egui::Sense::click());

                let tab_color = if is_active {
                    colors.section_active
                } else if response.hovered() {
                    colors.tab_hover
                } else {
                    colors.section_inactive
                };

                ui.painter().rect_filled(tab_rect, egui::Rounding::same(6.0), tab_color);
                ui.painter().rect_stroke(tab_rect, egui::Rounding::same(6.0), egui::Stroke::new(2.0, colors.leather));

                // For Perks tab, show small naturalist badge icon + text
                if *section == JournalSection::Perks {
                    if let Some(badge_id) = textures.get_naturalist_badge() {
                        let icon_size = 24.0;
                        let icon_rect = egui::Rect::from_min_size(
                            egui::pos2(tab_rect.min.x + 8.0, tab_rect.center().y - icon_size / 2.0),
                            egui::vec2(icon_size, icon_size),
                        );
                        ui.painter().image(badge_id, icon_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                        ui.painter().text(
                            egui::pos2(tab_rect.center().x + 10.0, tab_rect.center().y),
                            egui::Align2::CENTER_CENTER,
                            section.name(),
                            egui::FontId::proportional(15.0),
                            colors.ink,
                        );
                    } else {
                        ui.painter().text(tab_rect.center(), egui::Align2::CENTER_CENTER, section.name(), egui::FontId::proportional(16.0), colors.ink);
                    }
                } else {
                    ui.painter().text(tab_rect.center(), egui::Align2::CENTER_CENTER, section.name(), egui::FontId::proportional(16.0), colors.ink);
                }

                if response.clicked() {
                    state.active_section = *section;
                    state.active_inner_tab = 0;
                    state.selected_item = None;
                }
            }

            // === SECTION EMBLEM (large badge for Perks) ===
            let emblem_y = section_tab_y + section_tab_height + 15.0;
            let header_y;
            if state.active_section == JournalSection::Perks {
                if let Some(badge_id) = textures.get_naturalist_badge() {
                    let emblem_size = 80.0;
                    let emblem_rect = egui::Rect::from_min_size(
                        egui::pos2(journal_width / 2.0 - emblem_size / 2.0, emblem_y),
                        egui::vec2(emblem_size, emblem_size),
                    );
                    ui.painter().image(badge_id, emblem_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                    header_y = emblem_y + emblem_size + 10.0;
                } else {
                    header_y = emblem_y + 5.0;
                }
            } else {
                header_y = emblem_y + 5.0;
            }

            // === CATEGORY HEADER ===
            let category_name = match state.active_section {
                JournalSection::Perks => state.active_perk_tree.name(),
                JournalSection::Stats => state.active_stat_category.name(),
                JournalSection::Encyclopedia => state.active_encyclopedia.name(),
            };
            ui.painter().text(
                egui::pos2(journal_width / 2.0, header_y),
                egui::Align2::CENTER_CENTER,
                category_name.to_uppercase(),
                egui::FontId::proportional(22.0),
                colors.ink,
            );

            // === INNER TABS (below header) - only for Perks section ===
            let content_top;
            if state.active_section == JournalSection::Perks {
                let inner_tab_y = header_y + 30.0;
                let inner_tabs = state.active_perk_tree.inner_tabs();
                let inner_tab_width = 90.0;
                let inner_tab_height = 26.0;
                let inner_spacing = 6.0;
                let total_inner = (inner_tabs.len() as f32) * inner_tab_width + ((inner_tabs.len() - 1) as f32) * inner_spacing;
                let inner_start_x = (journal_width - total_inner) / 2.0;

                for (i, tab_name) in inner_tabs.iter().enumerate() {
                    let tab_x = inner_start_x + (i as f32) * (inner_tab_width + inner_spacing);
                    let tab_rect = egui::Rect::from_min_size(
                        egui::pos2(tab_x, inner_tab_y),
                        egui::vec2(inner_tab_width, inner_tab_height),
                    );

                    let is_active = state.active_inner_tab == i;
                    let response = ui.allocate_rect(tab_rect, egui::Sense::click());

                    let tab_color = if is_active {
                        colors.tab_active
                    } else if response.hovered() {
                        colors.tab_hover
                    } else {
                        colors.tab_inactive
                    };

                    ui.painter().rect_filled(tab_rect, egui::Rounding::same(4.0), tab_color);
                    ui.painter().rect_stroke(tab_rect, egui::Rounding::same(4.0), egui::Stroke::new(1.0, colors.leather));
                    ui.painter().text(
                        tab_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *tab_name,
                        egui::FontId::proportional(12.0),
                        colors.ink,
                    );

                    if response.clicked() {
                        state.active_inner_tab = i;
                        state.selected_item = None;
                    }
                }
                content_top = inner_tab_y + inner_tab_height + 15.0;
            } else {
                content_top = header_y + 40.0;
            }

            // === CONTENT AREA ===
            let content_height = journal_height - content_top - paper_margin - 20.0;
            let page_width = (journal_width / 2.0) - paper_margin - 20.0;
            let left_page_x = paper_margin + 15.0;
            let right_page_x = journal_width / 2.0 + 15.0;

            let left_rect = egui::Rect::from_min_size(egui::pos2(left_page_x, content_top), egui::vec2(page_width, content_height));
            let right_rect = egui::Rect::from_min_size(egui::pos2(right_page_x, content_top), egui::vec2(page_width, content_height));

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
            }

            // === CLOSE BUTTON ===
            let close_rect = egui::Rect::from_min_size(egui::pos2(journal_width - 40.0, 5.0), egui::vec2(30.0, 30.0));
            let close_response = ui.allocate_rect(close_rect, egui::Sense::click());
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                "X",
                egui::FontId::proportional(20.0),
                if close_response.hovered() { Color32::WHITE } else { colors.paper },
            );
            if close_response.clicked() {
                state.is_open = false;
            }
        });
}

// ============================================================================
// EMBLEM TAB RENDERER (Returns clicked index)
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
