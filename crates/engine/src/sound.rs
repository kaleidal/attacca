use crate::dsp::eq::EqCurve;
use crate::dsp::filters::FilterSettings;

/// The sound every song gets unless it has its own.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Sound {
    pub eq: EqCurve,
    pub filters: FilterSettings,
}

/// One song's level and its own equalizer and filters, which replace the global ones.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TrackSound {
    pub gain: f32,
    pub eq: Option<EqCurve>,
    pub filters: Option<FilterSettings>,
}

impl Default for TrackSound {
    fn default() -> Self {
        Self {
            gain: 1.0,
            eq: None,
            filters: None,
        }
    }
}
