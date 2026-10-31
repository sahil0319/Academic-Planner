//! One module per page (research R-D6). Each has `refresh(&App)`, called by
//! `App::refresh_all`, and `wire(&Rc<App>)`, called once at startup to connect callbacks.

pub mod about;
pub mod assessments;
pub mod courses;
pub mod dashboard;
pub mod import;
pub mod progress;
pub mod public_holidays;
pub mod rules;
pub mod settings;
pub mod timetable;
