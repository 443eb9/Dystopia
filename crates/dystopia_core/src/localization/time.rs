use crate::{
    localization::LocalizableEnum,
    sci::unit::{Time, Unit, TICKS_PER_SEC},
};

impl LocalizableEnum for Time {
    fn localize(&self, _lang: &super::LangFile) -> String {
        let total_secs = (self.to_si() / TICKS_PER_SEC) as f64;
        let hours = total_secs / 3600.;
        let mins = hours.fract() * 60.;
        let secs = mins.fract() * 60.;

        let i_hours = hours as i32;
        let i_mins = mins as i32;
        let i_secs = secs as i32;

        if i_hours == 0 {
            if i_mins == 0 {
                if i_secs == 0 {
                    "Now".into()
                } else {
                    format!("{}s", i_secs)
                }
            } else {
                format!("{}m {}s", i_mins, i_secs)
            }
        } else {
            format!("{}h {}m {}s", i_hours, i_mins, i_secs)
        }
    }
}
