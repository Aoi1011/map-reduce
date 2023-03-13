use std::{
    collections::HashMap,
    fs::{read_dir, File},
    io::BufReader,
    path::PathBuf,
    sync::Arc,
    thread,
};

use chan::{Receiver, Sender};

use crate::worker::{Job, JobResult, Worker};

pub struct Master {
    input_files: Vec<PathBuf>,
    working_directory: PathBuf,
    map: Arc<dyn Fn(BufReader<File>) -> Vec<String> + Send + Sync>,
    reduce: Arc<dyn Fn(Vec<BufReader<File>>) -> String + Send + Sync>,
    job_queue: Sender<Job>,
    results_queue: Receiver<JobResult>,
    worker_job_queue: Receiver<Job>,
    worker_results_queue: Sender<JobResult>,
}

impl Master {
    pub fn new(
        working_directory: PathBuf,
        input_files: Vec<PathBuf>,
        map: Arc<dyn Fn(BufReader<File>) -> Vec<String> + Send + Sync>,
        reduce: Arc<dyn Fn(Vec<BufReader<File>>) -> String + Send + Sync>,
    ) -> Self {
        let (work_send, work_recv) = chan::r#async();
        let (result_send, result_recv) = chan::r#async();

        Master {
            input_files,
            working_directory,
            map,
            reduce,
            job_queue: work_send,
            results_queue: result_recv,
            worker_job_queue: work_recv,
            worker_results_queue: result_send,
        }
    }

    fn do_map(&self) -> i32 {
        for (index, input) in self.input_files.iter().enumerate() {
            self.job_queue
                .send(Job::Map(((index + 1) as i32, input.clone())));
        }

        self.input_files.iter().len() as i32
    }

    fn do_reduce(&self) -> i32 {
        if let Ok(entries) = read_dir(self.working_directory.clone()) {
            let groups = entries.filter_map(|entry| entry.ok()).fold(
                HashMap::new(),
                |mut grouped, entry| {
                    let _ = entry
                        .file_name()
                        .into_string()
                        .and_then(|filename| {
                            filename
                                .split(".")
                                .nth(3)
                                .and_then(|i| i.parse().ok())
                                .ok_or(entry.file_name())
                        })
                        .map(|key| {
                            let mut files = grouped.entry(key).or_insert(vec![]);
                            files.push(entry.path())
                        });
                    grouped
                },
            );
            let n_reduce_jobs = groups.iter().len();
            for (index, group) in groups {
                self.job_queue.send(Job::Reduce((index, group)));
            }
            n_reduce_jobs as i32
        } else {
            0
        }
    }

    pub fn run(&self, n_workers: i32) -> Vec<PathBuf> {
        self.spawn_workers(n_workers);

        let n_map = self.do_map();
        self.wait_for_completion(n_map);
        let n_reduce = self.do_reduce();
        self.wait_for_completion(n_reduce);

        self.argument_result_files()
    }

    fn spawn_workers(&self, n_workers: i32) {
        for _ in 0..n_workers {
            let working_directory = self.working_directory.clone();
            let map = self.map.clone();
            let reduce = self.reduce.clone();
            let job_queue = self.worker_job_queue.clone();
            let results_queue = self.worker_results_queue.clone();

            thread::spawn(move || {
                let worker = Worker {
                    working_directory,
                    map,
                    reduce,
                    job_queue,
                    results_queue,
                };
                worker.run()
            });
        }
    }

    fn wait_for_completion(&self, n_jobs: i32) {
        let mut n_complete = 0;
        while n_complete < n_jobs {
            self.results_queue.recv().map(|_| n_complete += 1);
        }
    }

    fn argument_result_files(&self) -> Vec<PathBuf> {
        read_dir(self.working_directory.clone())
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        entry.file_name().into_string().ok().and_then(|name| {
                            match name.split(".").last() {
                                Some("result") => Some(entry),
                                _ => None,
                            }
                        })
                    })
                    .map(|entry| entry.path())
                    .collect()
            })
            .unwrap_or(vec![])
    }
}
