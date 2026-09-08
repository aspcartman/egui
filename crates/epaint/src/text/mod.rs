//! Everything related to text, fonts, text layout, cursors etc.

pub mod cursor;
mod face_store;
mod family;
mod font_data;
mod font_definitions;
mod font_face;
mod font_id;
mod font_provider;
mod font_tweak;
mod fonts;
mod galley_cache;
mod glyph_atlas;
mod glyph_rasterizer;
mod index;
mod styled_metrics;
mod text_layout;
mod text_layout_types;
mod unicode;

pub use {
    font_data::{Blob, FontData, FontVariationAxis},
    font_definitions::{FontDefinitions, FontInsert, FontPriority, InsertFontFamily},
    font_id::{FontFamily, FontId},
    font_provider::{FallbackRequest, FontProvider},
    font_tweak::{FontTweak, HintingTarget, SmoothHinting},
    fonts::{Fonts, FontsView, MAX_GLYPH_SIZE},
    glyph_rasterizer::{
        GlyphBitmap, GlyphRasterizer, GlyphRasterizerRequest, RasterizedGlyph,
        has_emoji_presentation,
    },
    index::{ByteIndex, ByteRange, ByteRangeExt, CharIndex, CharRange, CharRangeExt},
    text_layout_types::*,
};

/// Suggested character to use to replace those in password text fields.
pub const PASSWORD_REPLACEMENT_CHAR: char = '•';

/// Controls how we render text
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TextOptions {
    /// Maximum size of the font texture.
    pub max_texture_side: usize,

    /// Controls how to convert glyph colors when writing to the font atlas.
    pub color_transfer_function: crate::FontColorTransferFunction,

    /// Whether to enable font hinting
    ///
    /// (round some font coordinates to pixels for sharper text).
    ///
    /// Default is `true`.
    pub font_hinting: bool,

    /// Outline expansion radius in physical pixels, independent of font size and DPI.
    ///
    /// Thickens outline glyphs without changing layout. Color glyphs and platform
    /// fallback bitmaps are unaffected. Try `0.15` for subtle thickening.
    /// Clamped to `0.0..=1.0`; non-finite values disable it. Default: `0.0`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub glyph_dilation: f32,

    /// Scale dilation by text brightness in five levels, from zero for black to
    /// `glyph_dilation` for white. Uses GPUI's brightness rule, not CoreGraphics'
    /// proprietary rasterization. Default: false (fixed dilation).
    #[cfg_attr(feature = "serde", serde(default))]
    pub glyph_dilation_by_brightness: bool,

    /// Enable sub-pixel binning for glyphs.
    ///
    /// Sub-pixel binning renders each glyph at up to four fractional horizontal offsets,
    /// giving more even kerning at the cost of more atlas space.
    ///
    /// It also lead to text looking more blurry.
    ///
    /// This is always disabled for CJK characters (which have too many unique glyphs).
    ///
    /// Can be overridden per font with [`FontTweak::subpixel_binning`].
    ///
    /// Default: `true`.
    pub subpixel_binning: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            max_texture_side: 2048, // Small but portable
            color_transfer_function: crate::FontColorTransferFunction::default(),
            font_hinting: true,
            glyph_dilation: 0.0,
            glyph_dilation_by_brightness: false,
            subpixel_binning: true,
        }
    }
}

impl TextOptions {
    /// Select once per section; five cached levels follow GPUI's brightness quantization.
    pub(crate) fn dilation_level(&self, color: ecolor::Color32) -> u8 {
        if !self.glyph_dilation.is_finite() || self.glyph_dilation <= 0.0 {
            return 0;
        }
        if !self.glyph_dilation_by_brightness {
            return 4;
        }
        let [r, g, b, _] = color.to_srgba_unmultiplied();
        let brightness = (0.2126 * f32::from(r) + 0.7152 * f32::from(g)
            + 0.0722 * f32::from(b)) / 255.0;
        (brightness * 4.0 + 0.5).floor().clamp(0.0, 4.0) as u8
    }
}
