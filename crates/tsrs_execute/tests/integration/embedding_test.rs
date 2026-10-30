// The helpers an embedder of tsrs's crates calls to report, trace and map content as the `tsrs` CLI does stay
// public.

#[test]
fn the_cli_reporting_helpers_are_public() {
    let _ = tsrs_execute::tsc::create_diagnostic_reporter;
    let _ = tsrs_execute::tsc::create_diagnostic_formatter;
    let _ = tsrs_execute::tsc::create_report_error_summary;
    let _ = tsrs_execute::tsc::create_watch_status_reporter;
    let _ = tsrs_execute::tsc::get_trace_with_writer_from_sys;
    let _ = tsrs_execute::tsc::new_content_mapper_host;
}
