use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::PathBuf,
    sync::Arc,
};

use chan::{Receiver, Sender};

pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub enum Job {
    Map((i32, PathBuf)),
    Reduce((i32, Vec<PathBuf>)),
}

#[derive(Debug, PartialEq)]
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

impl Worker {
    pub fn run(&self) {
        for job in self.job_queue.iter() {
            match job {
                Job::Map((job_id, path)) => {
                    let results = (self.map)(open_file(path));
                    let names = self.map_result_names(&job_id, results.len());
                    self.write_map_results(names, results);
                    self.results_queue.send(JobResult::MapFinished(job_id));
                }
                Job::Reduce((job_id, paths)) => {
                    let files = paths
                        .into_iter()
                        .map(|path| open_file(path))
                        .collect::<Vec<BufReader<File>>>();
                    let result = (self.reduece)(files);
                    let name = self.reduce_result_name(&job_id);
                    self.write_reduce_results(name, result);
                    self.results_queue.send(JobResult::ReduceFinished(job_id));
                }
            }
        }
    }

    fn map_result_names(&self, job_id: &i32, length: usize) -> Vec<PathBuf> {
        (1..length)
            .map(|i| {
                let mut path = self.working_directory.clone();
                path.push(format!("map.{}.reduce.{}", job_id, i));
                path
            })
            .collect()
    }

    fn write_map_results(&self, names: Vec<PathBuf>, results: Vec<String>) {
        for (filename, result) in names.iter().zip(results.into_iter()) {
            let mut f = File::create(filename).unwrap();
            let _ = f.write_all(result.as_bytes());
        }
    }

    fn reduce_result_name(&self, job_id: &i32) -> PathBuf {
        let mut path = self.working_directory.clone();
        path.push(format!("reduce.{}.result", job_id));
        path
    }

    fn write_reduce_results(&self, name: PathBuf, result: String) {
        let mut f = File::create(name).unwrap();
        let _ = f.write_all(&result.as_bytes());
    }
}

fn open_file(path: PathBuf) -> BufReader<File> {
    let f = OpenOptions::new().read(true).open(path).unwrap();
    BufReader::new(f)
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    fn map_fn(input: BufReader<File>) -> Vec<String> {
        vec!["1", "2", "3", "4"].iter().map(|s| s.to_string()).collect()
    }

    fn reduce_fn(input: Vec<BufReader<File>>) -> String {
        "1234".to_string()
    }

    #[test]
    fn worker_maps_input_to_result_files() {
        let working_directry = PathBuf::from("./test-data/worker_maps_input_to_result_files");
        let mut map_file = working_directry.clone();
        map_file.push("input_file");

        let (work_send, work_recv) = chan::r#async();
        let (results_send, results_recv) = chan::r#async();

        let worker = Worker {
            working_directory: working_directry.clone(),
            map: Arc::new(map_fn),
            reduece: Arc::new(reduce_fn),
            job_queue: work_recv,
            results_queue: results_send,
        };

        thread::spawn(move || {
            worker.run()
        });

        work_send.send(Job::Map((1, map_file.clone())));
        let done = results_recv.recv();
        drop(work_send);
        drop(results_recv);

        assert_eq!(done, Some(JobResult::MapFinished(1)));
    }
}
