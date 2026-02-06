// Define system metrics as static observables

use std::sync::{Arc, Mutex};

use opentelemetry::KeyValue;
use sysinfo::System;

use crate::meter::get_meter;

pub struct SystemMetrics {
    sys: Mutex<System>,
    _counter_registration: opentelemetry::metrics::ObservableGauge<f64>,
    system_stats: Arc<Mutex<SystemStats>>,
}

#[derive(Clone)]
struct SystemStats {
    cpu_usage: f64,
    memory_usage_mb: f64,
}

impl SystemMetrics {
    pub fn new() -> Self {
        let meter = get_meter();
        let system_stats = Arc::new(Mutex::new(SystemStats {
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
        }));

        let stats_for_callback = system_stats.clone();

        let sys = System::new_all();

        let system_gauge = meter
            .f64_observable_gauge("system_resource_usage")
            .with_description("System resource utilization")
            .with_callback(move |observer| {
                // Note: Observable callbacks are sync, so we use try_lock
                if let Ok(stats) = stats_for_callback.try_lock() {
                    observer.observe(
                        stats.cpu_usage,
                        &[
                            KeyValue::new("resource", "cpu"),
                            KeyValue::new("unit", "percent"),
                        ],
                    );
                    observer.observe(
                        stats.memory_usage_mb,
                        &[
                            KeyValue::new("resource", "memory"),
                            KeyValue::new("unit", "megabytes"),
                        ],
                    );
                }
            })
            .build();

        Self {
            sys: Mutex::new(sys),
            _counter_registration: system_gauge,
            system_stats,
        }
    }

    pub async fn update_stats(&self) {
        let mut stats = self.system_stats.lock().unwrap();
        self.refresh();
        stats.cpu_usage = self.cpu_usage();
        stats.memory_usage_mb = self.memory_usage() as f64;
    }

    fn refresh(&self) {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_all();
    }

    fn cpu_usage(&self) -> f64 {
        let sys = self.sys.lock().unwrap();
        sys.global_cpu_usage() as f64
    }

    fn memory_usage(&self) -> u64 {
        let sys = self.sys.lock().unwrap();
        sys.used_memory()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self::new()
    }
}
