// Copyright 2024 Jeremy Edwards
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use log::warn;

pub(crate) fn worker_consumer<T, R, F>(
    name: &str,
    max_workers: usize,
    tx: crossbeam_channel::Sender<T>,
    rx: &crossbeam_channel::Receiver<R>,
    job: F,
) -> impl FnOnce()
where
    F: FnOnce(crossbeam_channel::Sender<T>, crossbeam_channel::Receiver<R>) + Send + Copy + 'static,
    T: Send + 'static,
    R: Send + 'static,
{
    let num_workers = get_recommended_worker_count(max_workers);
    let pool = threadpool::Builder::new()
        .num_threads(num_workers)
        .thread_name(format!("worker-{name}"))
        .build();

    for _ in 0..num_workers {
        let t_tx = tx.clone();
        let t_rx = rx.clone();

        pool.execute(move || {
            job(t_tx, t_rx);
        });
    }

    move || {
        pool.join();
        drop(tx);
    }
}

fn get_recommended_worker_count(cap: usize) -> usize {
    let available = match std::thread::available_parallelism() {
        Ok(size) => size.get(),
        Err(error) => {
            warn!("failed to call available_parallelism, assuming 1 thread, {error}");
            1
        }
    };
    if available > cap {
        cap
    } else {
        available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts as ma;

    #[test]
    fn test_get_recommended_worker_count() {
        ma::assert_gt!(get_recommended_worker_count(4), 0);
        ma::assert_le!(get_recommended_worker_count(4), 4);
        assert_eq!(get_recommended_worker_count(1), 1);
    }
}
