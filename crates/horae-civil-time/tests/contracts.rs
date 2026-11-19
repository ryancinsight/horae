//! Value-level checks for civil dates and timestamps.

mod contracts {
    use tracing_subscriber::fmt::format::FmtSpan;

    pub fn trace() -> tracing::subscriber::DefaultGuard {
        tracing::subscriber::set_default(
            tracing_subscriber::fmt()
                .with_span_events(FmtSpan::FULL)
                .with_test_writer()
                .finish(),
        )
    }

    mod calendar;
    mod timestamp;
}
