//! The worldgen progress feed — the port of `Terraria.WorldBuilding.
//! GenerationProgress` (`GenerationProgress.cs:1-92`), the object
//! `WorldGenerator` hands every pass (`WorldGenerator.cs:498-507`) and
//! `UIWorldLoad` reads (`UIWorldLoad.cs:69-91`).
//!
//! The weighted-progress math is verbatim: `TotalProgress` is
//! `(Value * CurrentPassWeight + _totalWeightedProgress) / TotalWeight`
//! (`:59-69`), `Start(weight)` resets the pass value (`:81-85`), `End()`
//! banks the pass weight (`:87-91`). The `Message` setter's `%` →
//! `{0:0.0%}` substitution (`:23`) and the getter's `string.Format`
//! (`:19`) are ported as [`GenerationProgress::message`] over
//! [`format_percent`].

/// .NET's custom numeric format `"0.0%"` as `GenerationProgress.Message`
/// and `UIWorldLoad.GetStatusText` (`UIWorldLoad.cs:133-135`) print it:
/// the value ×100, one decimal place, a `%` suffix. The custom-format
/// rounding is half-away-from-zero (`Math.Round`'s default is banker's,
/// but the `"0.0"` format specifier rounds midpoints away from zero).
pub fn format_percent(value: f64) -> String {
    let scaled = (value * 100.0 * 10.0).round() / 10.0;
    format!("{scaled:.1}%")
}

/// `GenerationProgress` (`GenerationProgress.cs:3-92`).
#[derive(Clone, Debug)]
pub struct GenerationProgress {
    /// `_message` (`:5`) — the raw template, `%` already substituted to
    /// `{0:0.0%}` by the setter (`:23`).
    message: String,
    /// `_value` (`:7`) — the current pass's 0..1 progress.
    value: f64,
    /// `_totalWeightedProgress` (`:9`) — the banked weight of every
    /// completed pass.
    total_weighted_progress: f64,
    /// `TotalWeight` (`:11`) — the sum of every enabled pass's weight
    /// (`WorldGenerator.cs:424`).
    pub total_weight: f64,
    /// `CurrentPassWeight` (`:13`, default 1.0).
    pub current_pass_weight: f64,
}

impl Default for GenerationProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationProgress {
    /// The C# field initializers: `_message = ""`, `CurrentPassWeight =
    /// 1.0` (`GenerationProgress.cs:5-13`), the rest zero.
    pub fn new() -> Self {
        GenerationProgress {
            message: String::new(),
            value: 0.0,
            total_weighted_progress: 0.0,
            total_weight: 0.0,
            current_pass_weight: 1.0,
        }
    }

    /// `Message`'s getter (`:15-20`): `string.Format(_message, Value)` —
    /// every `{0:0.0%}` token prints the current value as a percentage.
    pub fn message(&self) -> String {
        self.message.replace("{0:0.0%}", &format_percent(self.value))
    }

    /// `Message`'s setter (`:21-24`): `value.Replace("%", "{0:0.0%}")`.
    pub fn set_message(&mut self, value: &str) {
        self.message = value.replace('%', "{0:0.0%}");
    }

    /// `MessageNoFormatting`'s getter (`:27-32`) — the raw template.
    pub fn message_no_formatting(&self) -> &str {
        &self.message
    }

    /// `MessageNoFormatting`'s setter (`:33-36`) — no substitution.
    pub fn set_message_no_formatting(&mut self, value: &str) {
        self.message = value.to_string();
    }

    /// `Value`'s getter (`:39-44`).
    pub fn value(&self) -> f64 {
        self.value
    }

    /// `Value`'s setter (`:45-48`): `Utils.Clamp(value, 0.0, 1.0)`.
    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(0.0, 1.0);
    }

    /// `TotalWeightedProgress`'s setter (`:51-57`) — write-only in the C#.
    pub fn set_total_weighted_progress(&mut self, value: f64) {
        self.total_weighted_progress = value;
    }

    /// `TotalProgress` (`:59-69`): zero while `TotalWeight == 0`, else
    /// `(Value * CurrentPassWeight + _totalWeightedProgress) / TotalWeight`.
    pub fn total_progress(&self) -> f64 {
        if self.total_weight == 0.0 {
            return 0.0;
        }
        (self.value * self.current_pass_weight + self.total_weighted_progress) / self.total_weight
    }

    /// `Set(double value)` (`:71-74`).
    pub fn set(&mut self, value: f64) {
        self.set_value(value);
    }

    /// `Set(double value, double min, double max)` (`:76-79`):
    /// `Value = min + value * (max - min)` (the clamp rides the setter).
    pub fn set_range(&mut self, value: f64, min: f64, max: f64) {
        self.set_value(min + value * (max - min));
    }

    /// `Start(double weight)` (`:81-85`): the pass weight lands and the
    /// value resets — the raw field write, no clamp (the C# writes
    /// `_value` directly).
    pub fn start(&mut self, weight: f64) {
        self.current_pass_weight = weight;
        self.value = 0.0;
    }

    /// `End()` (`:87-91`): bank the pass weight, reset the value.
    pub fn end(&mut self) {
        self.total_weighted_progress += self.current_pass_weight;
        self.value = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TotalProgress`'s formula and zero-weight guard
    /// (`GenerationProgress.cs:59-69`).
    #[test]
    fn s1_e_17_total_progress_is_the_weighted_fraction() {
        let mut p = GenerationProgress::new();
        // TotalWeight == 0 → 0.0 regardless of the value.
        p.set(1.0);
        assert_eq!(p.total_progress(), 0.0);
        p.total_weight = 100.0;
        p.start(10.0);
        p.set(0.5);
        // (0.5 * 10 + 0) / 100
        assert_eq!(p.total_progress(), 0.05);
        p.set_total_weighted_progress(40.0);
        // (0.5 * 10 + 40) / 100
        assert_eq!(p.total_progress(), 0.45);
    }

    /// `Start`/`End`'s reset-and-bank pair (`:81-91`) — the
    /// `WorldGenerator.RunPass` bookends (`WorldGenerator.cs:498/:507`).
    #[test]
    fn s1_e_17_start_and_end_bank_the_pass_weight() {
        let mut p = GenerationProgress::new();
        p.total_weight = 30.0;
        p.start(10.0);
        assert_eq!(p.current_pass_weight, 10.0);
        assert_eq!(p.value(), 0.0);
        p.set(0.9);
        p.end();
        assert_eq!(p.value(), 0.0);
        assert_eq!(p.total_progress(), 10.0 / 30.0);
        p.start(20.0);
        p.set(0.25);
        // (0.25 * 20 + 10) / 30
        assert!((p.total_progress() - 0.5).abs() < 1e-12);
        p.end();
        assert_eq!(p.total_progress(), 1.0);
    }

    /// `Value`'s clamp (`:47`) and `Set(value, min, max)`'s remap (`:78`).
    #[test]
    fn s1_e_17_value_clamps_and_the_range_set_remaps() {
        let mut p = GenerationProgress::new();
        p.set(-0.5);
        assert_eq!(p.value(), 0.0);
        p.set(1.5);
        assert_eq!(p.value(), 1.0);
        p.set_range(0.5, 0.2, 0.8);
        assert!((p.value() - 0.5).abs() < 1e-12);
        // The remap clamps through the same setter.
        p.set_range(2.0, 0.2, 0.8);
        assert_eq!(p.value(), 1.0);
    }

    /// The `Message` setter's `%` substitution (`:23`) and the getter's
    /// `string.Format` (`:19`); `MessageNoFormatting` stays raw (`:27-36`).
    #[test]
    fn s1_e_17_message_formats_the_percent_token() {
        let mut p = GenerationProgress::new();
        p.set_message("Generating terrain (%)");
        assert_eq!(p.message_no_formatting(), "Generating terrain ({0:0.0%})");
        p.set(0.425);
        assert_eq!(p.message(), "Generating terrain (42.5%)");
        p.set_message_no_formatting("raw % stays");
        assert_eq!(p.message(), "raw % stays");
        // No token: the message passes through unchanged.
        p.set_message("Adding trees");
        assert_eq!(p.message(), "Adding trees");
    }

    /// `"0.0%"`'s one-decimal percentage shape, midpoints away from zero.
    #[test]
    fn s1_e_17_percent_format_matches_the_custom_specifier() {
        assert_eq!(format_percent(0.0), "0.0%");
        assert_eq!(format_percent(1.0), "100.0%");
        assert_eq!(format_percent(0.425), "42.5%");
        assert_eq!(format_percent(0.1234), "12.3%");
        assert_eq!(format_percent(0.1236), "12.4%");
        // 0.0005 → 0.05% → the midpoint rounds away from zero → 0.1%.
        assert_eq!(format_percent(0.0005), "0.1%");
    }
}
