use chrono::NaiveDate;

/// Controls what scale the timeline displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineScale {
    Days,
    Weeks,
    Months,
}

/// Manages the visible viewport of the timeline.
#[derive(Debug, Clone)]
pub struct TimelineViewport {
    /// The leftmost visible date.
    pub start: NaiveDate,
    /// The rightmost visible date.
    pub end: NaiveDate,
    /// Current display scale.
    pub scale: TimelineScale,
    /// Pixels per day (controls zoom level).
    pub pixels_per_day: f32,
}

impl TimelineViewport {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start,
            end,
            scale: TimelineScale::Weeks,
            pixels_per_day: 18.0,
        }
    }

    /// Convert a date to an x-pixel offset from the viewport start.
    pub fn date_to_x(&self, date: NaiveDate) -> f32 {
        let days = (date - self.start).num_days() as f32;
        days * self.pixels_per_day
    }

    /// Convert an x-pixel offset back to a date.
    pub fn x_to_date(&self, x: f32) -> NaiveDate {
        let days = (x / self.pixels_per_day).round() as i64;
        self.start + chrono::Duration::days(days)
    }

    /// Total width in pixels for the visible range.
    pub fn total_width(&self) -> f32 {
        self.date_to_x(self.end)
    }

    /// Zoom in (increase pixels per day).
    pub fn zoom_in(&mut self) {
        self.pixels_per_day = (self.pixels_per_day * 1.2).min(80.0);
    }

    /// Zoom out (decrease pixels per day).
    pub fn zoom_out(&mut self) {
        self.pixels_per_day = (self.pixels_per_day / 1.2).max(2.0);
    }

    /// Scroll the viewport by a number of days.
    pub fn scroll_days(&mut self, days: i64) {
        self.start += chrono::Duration::days(days);
        self.end += chrono::Duration::days(days);
    }
}
