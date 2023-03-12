use std::{path::PathBuf, sync::Arc, io::BufReader, fs::File};

use chan::{Receiver, Sender};

pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub enum Job {
    Map((i32, PathBuf)),
    Reduce((i32, Vec<PathBuf>)),
}

pub enum JobResult {
    MapFinished(i32),
    ReduceFinished(i32),
}

pub struct Worker {
    pub working_directory: PathBuf,
    pub map: Arc<dyn Fn(BufReader<File>) -> Vec<String> + Send + Sync>,
    pub reduece: Arc<dyn Fn(Vec<BufReader<File>>) -> String + Send + Sync>,
    pub job_queue: Receiver<Job>,
    pub results_queue: Sender<JobResult>,
}
