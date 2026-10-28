//! CSV exports (M3).

use std::io::Write;

use crate::db::Planner;
use crate::error::Result;

impl Planner {
    /// R23: writes `attendance.csv` (header row, sorted by date).
    pub fn export_attendance_csv<W: Write>(&self, w: W) -> Result<()> {
        let _ = w;
        todo!("M3: R23")
    }

    /// R23: writes `assessments.csv` (header row, sorted by date).
    pub fn export_assessments_csv<W: Write>(&self, w: W) -> Result<()> {
        let _ = w;
        todo!("M3: R23")
    }
}
