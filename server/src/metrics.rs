use std::{collections::HashMap, sync::{mpsc, Arc, LazyLock, RwLock, Mutex}, thread, time::{Duration, Instant}};

use crate::server::TimedStream;

static METRICS_CHANNEL: LazyLock<Arc<mpsc::Sender<MetricsMessage>>> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel::<MetricsMessage>();

    thread::Builder::new().name("metrics".into()).spawn(move || {
        take_metrics(receiver);
    }).expect("failed to create metrics thread: OS error");

    Arc::new(sender)
});

static METRIC_ID: LazyLock<RwLock<usize>> = LazyLock::new(|| {
    RwLock::new(0)
});

static STARTUP_TIME: Mutex<Option<Instant>> = Mutex::new(None);

pub struct MetricsMessage {
    thread_source: String,
    id: usize,
    checkpoint: Checkpoint
}
impl MetricsMessage {
    pub fn new(id: usize, checkpoint: Checkpoint) -> MetricsMessage {
        MetricsMessage {
            thread_source: thread_name(),
            id,
            checkpoint
        }
    }
}
impl std::fmt::Debug for MetricsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsMessage").field("thread_source", &self.thread_source).field("id", &self.id).field("checkpoint", &self.checkpoint).finish()
    }
}

#[derive(Debug)]
enum Checkpoint {
    Init,
    Start,
    Arrive,
    StreamClose,
    Leave
}

pub fn thread_name() -> String {
    thread::current().name().unwrap_or("anonymous").into()
}

pub fn thread_name_display() -> String {
    let binding = thread::current();
    format!("{} - {:?}", binding.name().unwrap_or("anonymous"), binding.id())
}

pub fn next_id() -> usize {
    let mut id = METRIC_ID.write().unwrap();
    let out = id.clone();
    *id += 1;
    out
}

pub fn begin_startup() {
    (*STARTUP_TIME.lock().unwrap()) = Some(Instant::now());
    METRICS_CHANNEL.send(MetricsMessage::new(0, Checkpoint::Init));
}

pub fn finish_startup() {
    println!("Startup Complete! - {:?}", (*STARTUP_TIME.lock().unwrap()).unwrap().elapsed());
}

pub fn start() -> usize {
    let id = next_id();
    METRICS_CHANNEL.send(MetricsMessage::new(id, Checkpoint::Start));
    id
}

pub fn arrive(id: usize) {
    METRICS_CHANNEL.send(MetricsMessage::new(id, Checkpoint::Arrive));
}

pub fn response_sent(id: usize) {
    METRICS_CHANNEL.send(MetricsMessage::new(id, Checkpoint::StreamClose));
}

pub fn end(id: usize) {
    METRICS_CHANNEL.send(MetricsMessage::new(id, Checkpoint::Leave));
}

pub struct Metric {
    pub start_time: Instant,
    pub end_time: Option<Duration>
}
impl Metric {
    pub fn new() -> Metric {
        Metric { start_time: Instant::now(), end_time: None }
    }
    pub fn end(&mut self) {
        self.end_time = Some(self.start_time.elapsed());
    }
    pub fn is_done(&self) -> bool {
        return self.end_time.is_some()
    }
}
impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.end_time.unwrap_or(Duration::new(0,0)))
    }
}
impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.end_time.unwrap_or(Duration::new(0,0)))
    }
}

struct StreamMetrics {
    id: usize,
    response_time: Metric,
    thread_times: HashMap<String, Metric>,
}
impl StreamMetrics {
    pub fn new(id: usize) -> StreamMetrics {
        StreamMetrics{ id, response_time: Metric::new(), thread_times: HashMap::new() }
    }
    pub fn thread_start(&mut self, name: String) {
        self.thread_times.insert(name, Metric::new());
    }
    pub fn thread_end(&mut self, name: String) {
        let Some(metric) = self.thread_times.get_mut(&name) else { 
            eprintln!("timer for thread {} attempted to end but doesnt exist !!", name);
            return
        };
        metric.end();
    }
    pub fn stream_close(&mut self) {
        self.response_time.end();
    }
    fn is_done(&self) -> bool {
        return self.response_time.is_done() && self.thread_times.iter().all(|(_, m)| m.is_done());
    }
}
impl std::fmt::Display for StreamMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tRequest #{} Latency: {}\n\tThread Latencies: {:?}", self.id, self.response_time, self.thread_times)
    }
}

fn take_metrics(receiver: mpsc::Receiver<MetricsMessage>) {

    println!("\t\tmetrics thread spawned:\t{}", thread_name_display());

    let mut timers: Vec<StreamMetrics> = Vec::new();

    for message in receiver.iter() {

        match message.checkpoint {
            Checkpoint::Init => {}
            Checkpoint::Start => {
                let stream_metric = StreamMetrics::new(message.id);
                timers.push(stream_metric);
            }
            Checkpoint::Arrive => {
                let Some(metrics) = timers.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} arrived at thread {} !!", message.id, message.thread_source);
                    continue;
                };
                metrics.thread_start(message.thread_source);
            }
            Checkpoint::StreamClose => {
                let Some(metrics) = timers.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} was written to from {} !!", message.id, message.thread_source);
                    continue;
                };
                metrics.stream_close();

                if metrics.is_done() {
                    println!("{}", metrics);
                }
            }
            Checkpoint::Leave => {
                let Some(metrics) = timers.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} left thread {} ??", message.id, message.thread_source);
                    continue;
                };
                metrics.thread_end(message.thread_source);

                if metrics.is_done() {
                    println!("{}", metrics);
                }
            }
        }
        
    }
}