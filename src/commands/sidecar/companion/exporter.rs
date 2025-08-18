use core::fmt;
use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::logs::LogBatch;
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData};
use opentelemetry_sdk::metrics::{
    data::{
        Gauge, GaugeDataPoint, Histogram, HistogramDataPoint, ResourceMetrics, ScopeMetrics, Sum,
        SumDataPoint,
    },
    exporter::PushMetricExporter,
};
use opentelemetry_sdk::trace::SpanData;
use std::fmt::Debug;
use std::sync::atomic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentelemetry_sdk::resource::Resource;

/// An OpenTelemetry exporter that writes Logs to stdout on export.
pub struct LogExporter {
    resource: Resource,
    is_shutdown: atomic::AtomicBool,
    resource_emitted: atomic::AtomicBool,
}

impl Default for LogExporter {
    fn default() -> Self {
        LogExporter {
            resource: Resource::builder().build(),
            is_shutdown: atomic::AtomicBool::new(false),
            resource_emitted: atomic::AtomicBool::new(false),
        }
    }
}

impl fmt::Debug for LogExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LogExporter")
    }
}

impl opentelemetry_sdk::logs::LogExporter for LogExporter {
    /// Export logs to memory
    async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Err(OTelSdkError::AlreadyShutdown)
        } else {
            println!("Logs");
            if self
                .resource_emitted
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                print_logs(batch);
            } else {
                if let Some(schema_url) = self.resource.schema_url() {
                    println!("\t Resource SchemaUrl: {schema_url:?}");
                }
                self.resource.iter().for_each(|(k, v)| {
                    println!("\t ->  {k}={v:?}");
                });
                print_logs(batch);
            }

            Ok(())
        }
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
    }
}

/// An OpenTelemetry exporter that writes to stdout on export.
pub struct MetricExporter {
    is_shutdown: atomic::AtomicBool,
    temporality: Temporality,
}

impl MetricExporter {
    /// Create a builder to configure this exporter.
    pub fn builder() -> MetricExporterBuilder {
        MetricExporterBuilder::default()
    }
}
impl Default for MetricExporter {
    fn default() -> Self {
        MetricExporterBuilder::default().build()
    }
}

impl fmt::Debug for MetricExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricExporter")
    }
}

impl PushMetricExporter for MetricExporter {
    /// Write Metrics to stdout
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        if self.is_shutdown.load(atomic::Ordering::SeqCst) {
            Err(opentelemetry_sdk::error::OTelSdkError::AlreadyShutdown)
        } else {
            println!("Metrics");
            if let Some(schema_url) = metrics.resource().schema_url() {
                println!("\tResource SchemaUrl: {schema_url:?}");
            }

            metrics.resource().iter().for_each(|(k, v)| {
                println!("\t ->  {k}={v:?}");
            });
            print_metrics(metrics.scope_metrics());
            Ok(())
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        // exporter holds no state, nothing to flush
        Ok(())
    }

    fn shutdown(&self) -> OTelSdkResult {
        self.shutdown_with_timeout(Duration::from_secs(5))
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        self.is_shutdown.store(true, atomic::Ordering::SeqCst);
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        self.temporality
    }
}

/// Configuration for the stdout metrics exporter
#[derive(Default)]
pub struct MetricExporterBuilder {
    temporality: Option<Temporality>,
}

impl MetricExporterBuilder {
    /// Set the [Temporality] of the exporter.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Create a metrics exporter with the current configuration
    pub fn build(self) -> MetricExporter {
        MetricExporter {
            temporality: self.temporality.unwrap_or_default(),
            is_shutdown: atomic::AtomicBool::new(false),
        }
    }
}

impl fmt::Debug for MetricExporterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MetricExporterBuilder")
    }
}

/// An OpenTelemetry exporter that writes Spans to stdout on export.
pub struct SpanExporter {
    resource: Resource,
    is_shutdown: AtomicBool,
    resource_emitted: AtomicBool,
}

impl fmt::Debug for SpanExporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SpanExporter")
    }
}

impl Default for SpanExporter {
    fn default() -> Self {
        SpanExporter {
            resource: Resource::builder().build(),
            is_shutdown: AtomicBool::new(false),
            resource_emitted: AtomicBool::new(false),
        }
    }
}

impl opentelemetry_sdk::trace::SpanExporter for SpanExporter {
    /// Write Spans to memory
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        if self.is_shutdown.load(Ordering::SeqCst) {
            Err(OTelSdkError::AlreadyShutdown)
        } else {
            println!("Spans");
            if self
                .resource_emitted
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                print_spans(batch);
            } else {
                if let Some(schema_url) = self.resource.schema_url() {
                    println!("\tResource SchemaUrl: {schema_url:?}");
                }

                self.resource.iter().for_each(|(k, v)| {
                    println!("\t ->  {k}={v:?}");
                });

                print_spans(batch);
            }

            Ok(())
        }
    }

    fn shutdown(&mut self) -> OTelSdkResult {
        self.is_shutdown.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn set_resource(&mut self, res: &opentelemetry_sdk::Resource) {
        self.resource = res.clone();
    }
}

fn print_logs(batch: LogBatch<'_>) {
    let mut field_storage = String::new();
    for (i, log) in batch.iter().enumerate() {
        field_storage.push_str(format!("Log #{i}").as_str());
        let (record, library) = log;

        field_storage.push_str(format!("\t Instrumentation Scope: {library:?}").as_str());

        if let Some(event_name) = record.event_name() {
            field_storage.push_str(format!("\t EventName: {event_name:?}").as_str());
        }
        if let Some(target) = record.target() {
            field_storage.push_str(format!("\t Target (Scope): {target:?}").as_str());
        }
        if let Some(trace_context) = record.trace_context() {
            field_storage.push_str(format!("\t TraceId: {:?}", trace_context.trace_id).as_str());
            field_storage.push_str(format!("\t SpanId: {:?}", trace_context.span_id).as_str());
            if let Some(trace_flags) = trace_context.trace_flags {
                field_storage.push_str(format!("\t TraceFlags: {trace_flags:?}").as_str());
            }
        }
        if let Some(severity) = record.severity_text() {
            field_storage.push_str(format!("\t SeverityText: {severity:?}").as_str());
        }
        if let Some(severity) = record.severity_number() {
            field_storage.push_str(format!("\t SeverityNumber: {severity:?}").as_str());
        }
        if let Some(body) = record.body() {
            field_storage.push_str(format!("\t Body: {body:?}").as_str());
        }

        field_storage.push_str("\t Attributes:");
        for (k, v) in record.attributes_iter() {
            field_storage.push_str(format!("\t\t ->  {k}: {v:?}").as_str());
        }
    }
    println!("{field_storage}");
}

fn print_spans(batch: Vec<SpanData>) {
    let mut field_storage = String::new();
    for (i, span) in batch.into_iter().enumerate() {
        field_storage.push_str(format!("Span #{i}").as_str());
        field_storage.push_str("\tInstrumentation Scope");
        field_storage.push_str(
            format!(
                "\t\tName         : {:?}",
                &span.instrumentation_scope.name()
            )
            .as_str(),
        );
        if let Some(version) = &span.instrumentation_scope.version() {
            field_storage.push_str(format!("\t\tVersion  : {version:?}").as_str());
        }
        if let Some(schema_url) = &span.instrumentation_scope.schema_url() {
            field_storage.push_str(format!("\t\tSchemaUrl: {schema_url:?}").as_str());
        }
        span.instrumentation_scope
            .attributes()
            .enumerate()
            .for_each(|(index, kv)| {
                if index == 0 {
                    field_storage.push_str("\t\tScope Attributes:");
                }
                field_storage.push_str(format!("\t\t\t ->  {}: {}", kv.key, kv.value).as_str());
            });

        field_storage.push_str(format!("\tName        : {}", &span.name).as_str());
        field_storage
            .push_str(format!("\tTraceId     : {}", &span.span_context.trace_id()).as_str());
        field_storage
            .push_str(format!("\tSpanId      : {}", &span.span_context.span_id()).as_str());
        field_storage
            .push_str(format!("\tTraceFlags  : {:?}", &span.span_context.trace_flags()).as_str());
        field_storage.push_str(format!("\tParentSpanId: {}", &span.parent_span_id).as_str());
        field_storage.push_str(format!("\tKind        : {:?}", &span.span_kind).as_str());

        field_storage.push_str(format!("\tStatus: {:?}", &span.status).as_str());

        let mut print_header = true;
        for kv in span.attributes.iter() {
            if print_header {
                field_storage.push_str("\tAttributes:");
                print_header = false;
            }
            field_storage.push_str(format!("\t\t ->  {}: {:?}", kv.key, kv.value).as_str());
        }

        span.events.iter().enumerate().for_each(|(index, event)| {
            if index == 0 {
                field_storage.push_str("\tEvents:");
            }
            field_storage.push_str(format!("\tEvent #{index}").as_str());
            field_storage.push_str(format!("\tName      : {}", event.name).as_str());

            event.attributes.iter().enumerate().for_each(|(index, kv)| {
                if index == 0 {
                    field_storage.push_str("\tAttributes:");
                }
                field_storage.push_str(format!("\t\t ->  {}: {:?}", kv.key, kv.value).as_str());
            });
        });

        span.links.iter().enumerate().for_each(|(index, link)| {
            if index == 0 {
                field_storage.push_str("\tLinks:");
            }
            field_storage.push_str(format!("\tLink #{index}").as_str());
            field_storage.push_str(format!("\tTraceId: {}", link.span_context.trace_id()).as_str());
            field_storage.push_str(format!("\tSpanId : {}", link.span_context.span_id()).as_str());

            link.attributes.iter().enumerate().for_each(|(index, kv)| {
                if index == 0 {
                    field_storage.push_str("\tAttributes:");
                }
                field_storage.push_str(format!("\t\t ->  {}: {:?}", kv.key, kv.value).as_str());
            });
        });
    }
    println!("{field_storage}");
}

fn print_metrics<'a>(metrics: impl Iterator<Item = &'a ScopeMetrics>) {
    let mut field_storage = String::new();
    for (i, metric) in metrics.enumerate() {
        field_storage.push_str(format!("\tInstrumentation Scope #{i}").as_str());
        let scope = metric.scope();
        field_storage.push_str(format!("\t\tName         : {}", scope.name()).as_str());
        if let Some(version) = scope.version() {
            field_storage.push_str(format!("\t\tVersion  : {version:?}").as_str());
        }
        if let Some(schema_url) = scope.schema_url() {
            field_storage.push_str(format!("\t\tSchemaUrl: {schema_url:?}").as_str());
        }
        scope.attributes().enumerate().for_each(|(index, kv)| {
            if index == 0 {
                field_storage.push_str(format!("\t\tScope Attributes:").as_str());
            }
            field_storage.push_str(format!("\t\t\t ->  {}: {}", kv.key, kv.value).as_str());
        });

        metric.metrics().enumerate().for_each(|(i, metric)| {
            field_storage.push_str(format!("Metric #{i}").as_str());
            field_storage.push_str(format!("\t\tName         : {}", metric.name()).as_str());
            field_storage.push_str(format!("\t\tDescription  : {}", metric.description()).as_str());
            field_storage.push_str(format!("\t\tUnit         : {}", metric.unit()).as_str());

            fn print_info<T>(data: &MetricData<T>, s: &mut String)
            where
                T: Debug + Copy,
            {
                match data {
                    MetricData::Gauge(gauge) => {
                        s.push_str("\t\tType         : Gauge");
                        print_gauge(gauge, s);
                    }
                    MetricData::Sum(sum) => {
                        s.push_str("\t\tType         : Sum");
                        print_sum(sum, s);
                    }
                    MetricData::Histogram(hist) => {
                        s.push_str("\t\tType         : Histogram");
                        print_histogram(hist, s);
                    }
                    MetricData::ExponentialHistogram(_) => {
                        s.push_str("\t\tType         : Exponential Histogram");
                        // TODO: add support for ExponentialHistogram
                    }
                }
            }
            match metric.data() {
                AggregatedMetrics::F64(data) => print_info(data, &mut field_storage),
                AggregatedMetrics::U64(data) => print_info(data, &mut field_storage),
                AggregatedMetrics::I64(data) => print_info(data, &mut field_storage),
            }
        });
    }
    println!("{field_storage}");
}

fn print_sum<T: Debug + Copy>(sum: &Sum<T>, s: &mut String) {
    s.push_str("\t\tSum DataPoints");
    s.push_str(format!("\t\tMonotonic    : {}", sum.is_monotonic()).as_str());
    if sum.temporality() == Temporality::Cumulative {
        s.push_str("\t\tTemporality  : Cumulative");
    } else {
        s.push_str("\t\tTemporality  : Delta");
    }
    print_sum_data_points(sum.data_points(), s);
}

fn print_gauge<T: Debug + Copy>(gauge: &Gauge<T>, s: &mut String) {
    s.push_str("\t\tGauge DataPoints");
    print_gauge_data_points(gauge.data_points(), s);
}

fn print_histogram<T: Debug + Copy>(histogram: &Histogram<T>, s: &mut String) {
    if histogram.temporality() == Temporality::Cumulative {
        s.push_str("\t\tTemporality  : Cumulative");
    } else {
        s.push_str("\t\tTemporality  : Delta");
    }
    s.push_str("\t\tHistogram DataPoints");
    print_hist_data_points(histogram.data_points(), s);
}

fn print_sum_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a SumDataPoint<T>>,
    s: &mut String,
) {
    for (i, data_point) in data_points.enumerate() {
        s.push_str(format!("\t\tDataPoint #{i}").as_str());
        s.push_str(format!("\t\t\tValue        : {:#?}", data_point.value()).as_str());
        s.push_str("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            s.push_str(format!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str()).as_str());
        }
    }
}

fn print_gauge_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a GaugeDataPoint<T>>,
    s: &mut String,
) {
    for (i, data_point) in data_points.enumerate() {
        s.push_str(format!("\t\tDataPoint #{i}").as_str());
        s.push_str(format!("\t\t\tValue        : {:#?}", data_point.value()).as_str());
        s.push_str("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            s.push_str(format!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str()).as_str());
        }
    }
}

fn print_hist_data_points<'a, T: Debug + Copy + 'a>(
    data_points: impl Iterator<Item = &'a HistogramDataPoint<T>>,
    s: &mut String,
) {
    for (i, data_point) in data_points.enumerate() {
        s.push_str(format!("\t\tDataPoint #{i}").as_str());
        s.push_str(format!("\t\t\tCount        : {}", data_point.count()).as_str());
        s.push_str(format!("\t\t\tSum          : {:?}", data_point.sum()).as_str());
        if let Some(min) = &data_point.min() {
            s.push_str(format!("\t\t\tMin          : {min:?}").as_str());
        }

        if let Some(max) = &data_point.max() {
            s.push_str(format!("\t\t\tMax          : {max:?}").as_str());
        }

        s.push_str("\t\t\tAttributes   :");
        for kv in data_point.attributes() {
            s.push_str(format!("\t\t\t\t ->  {}: {}", kv.key, kv.value.as_str()).as_str());
        }

        let mut lower_bound = f64::NEG_INFINITY;
        let bounds_iter = data_point.bounds();
        let mut bucket_counts_iter = data_point.bucket_counts();
        let mut header_printed = false;

        // Process all the regular buckets
        for upper_bound in bounds_iter {
            // Print header only once before the first item
            if !header_printed {
                s.push_str("\t\t\tBuckets");
                header_printed = true;
            }

            // Get the count for this bucket, or 0 if not available
            let count = bucket_counts_iter.next().unwrap_or(0);
            s.push_str(format!("\t\t\t\t {lower_bound} to {upper_bound} : {count}").as_str());
            lower_bound = upper_bound;
        }

        // Handle the final +Infinity bucket if we processed any buckets
        if header_printed {
            let last_count = bucket_counts_iter.next().unwrap_or(0);
            s.push_str(format!("\t\t\t\t{lower_bound} to +Infinity : {last_count}").as_str());
        }
    }
}
