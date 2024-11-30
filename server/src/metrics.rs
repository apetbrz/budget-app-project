use std::{collections::HashMap, ops::Add, sync::{mpsc, Arc, LazyLock, Mutex, RwLock}, thread, time::{Duration, Instant}};

use colored::Colorize;
use http_bytes::http::StatusCode;

use crate::{http_utils, server::TimedStream};


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
    Leave,
    Query(TimedStream)
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

pub fn query(stream: TimedStream) {
    METRICS_CHANNEL.send(MetricsMessage::new(0, Checkpoint::Query(stream)));
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
    real_time: Metric,
    thread_times: HashMap<String, Metric>,
}
impl StreamMetrics {
    pub fn new(id: usize) -> StreamMetrics {
        StreamMetrics{ id, response_time: Metric::new(), real_time: Metric::new(), thread_times: HashMap::new() }
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
        write!(f, "{}{}\tRequest Latency: {}\n\t\tProcessor Latency: {}\n\t\tThread Latencies: {:?}\n",
        "  * ".bright_yellow().bold(), self.id, self.response_time, self.real_time, self.thread_times)
    }
}

fn take_metrics(receiver: mpsc::Receiver<MetricsMessage>) {

    println!("\t\tmetrics thread spawned:\t{}", thread_name_display());

    let mut stream_data: Vec<StreamMetrics> = Vec::new();

    for message in receiver.iter() {

        match message.checkpoint {
            Checkpoint::Init => {}
            Checkpoint::Start => {
                let stream_metric = StreamMetrics::new(message.id);
                stream_data.push(stream_metric);
            }
            Checkpoint::Arrive => {
                let Some(metrics) = stream_data.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} arrived at thread {} !!", message.id, message.thread_source);
                    continue;
                };
                metrics.thread_start(message.thread_source);
            }
            Checkpoint::StreamClose => {
                let Some(metrics) = stream_data.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} was written to from {} !!", message.id, message.thread_source);
                    continue;
                };
                metrics.stream_close();

                if metrics.is_done() {
                    metrics.real_time.end();
                    println!("{}", metrics);
                }
            }
            Checkpoint::Leave => {
                let Some(metrics) = stream_data.get_mut(message.id) else {
                    eprintln!("stream that doesnt exist {} left thread {} ??", message.id, message.thread_source);
                    continue;
                };
                metrics.thread_end(message.thread_source);

                if metrics.is_done() {
                    metrics.real_time.end();
                    println!("{}", metrics);
                }
            }
            Checkpoint::Query(mut stream) => {
                let out = stream_data.iter().fold(
                    (Duration::new(0,0), Duration::new(0,0), HashMap::<String, (u32, Duration)>::new()),
                    |(mut res, mut cpu, mut threads), m| {

                        if m.response_time.end_time.is_none() || m.real_time.end_time.is_none() { return (res, cpu, threads); }
                        
                        res += m.response_time.end_time.unwrap();
                        cpu += m.real_time.end_time.unwrap();

                        //TODO: FIX THIS AVERAGE: IGNORES HOW MANY TIMES EACH THREAD ACTUALLY SHOWS UP
                        //HashMap<String, (usize, Duration)> ???
                        m.thread_times.iter()
                        .for_each(|(label, metric)| {
                            if metric.end_time.is_none() { return; }
                            let entry = threads.entry(label.clone()).or_insert((1, metric.end_time.unwrap()));
                            entry.0 += 1;
                            entry.1 += metric.end_time.unwrap();
                        });

                        (res, cpu, threads)
                    }
                );

                let avg_divisor = stream_data.len() as f32;

                let thread_latencies: HashMap<String, String> = out.2.iter().map(|(l, d)| (l.clone(), format!("{:?}", d.1.div_f32(d.0 as f32)))).collect();

                let output = format!("{{\"average_response_latency\":\"{:?}\",\"average_processor_latency\":\"{:?}\",\"average_thread_latencies\":{:?}}}", out.0.div_f32(avg_divisor), out.1.div_f32(avg_divisor), thread_latencies);
                
                http_utils::send_response(http_utils::ok_json(StatusCode::OK, output).unwrap(), &mut stream);

            }
        }
        
    }
}