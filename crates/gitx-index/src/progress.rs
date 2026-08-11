#[derive(Debug, Default, Clone)]
pub struct Progress {
    pub objects_discovered: usize,
    pub commits_processed: usize,
    pub files_processed: usize,
    pub changes_processed: usize,
}

pub trait ProgressReporter {
    fn report(&mut self, progress: &Progress);
}

pub struct NoopProgress;
impl ProgressReporter for NoopProgress {
    fn report(&mut self, _progress: &Progress) {}
}
