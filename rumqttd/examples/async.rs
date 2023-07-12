use rumqttd::{Broker, Config, Error, Spawner};

use std::{fmt::Display, future::Future, time::Duration};

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    pretty_env_logger::init();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());
}

async fn run() {
    // As examples are compiled as seperate binary so this config is current path dependent. Run it
    // from root of this crate
    let config = config::Config::builder()
        .add_source(config::File::with_name("rumqttd.toml"))
        .build()
        .unwrap();

    let config: Config = config.try_deserialize().unwrap();

    let mut broker = Broker::new(config);

    let (mut link_tx, mut link_rx) = broker.link("local").unwrap();
    link_tx.subscribe("#").unwrap();

    let handle = tokio::task::spawn(async move {
        loop {
            let notification = match link_rx.next().await.unwrap() {
                Some(v) => v,
                None => continue,
            };

            println!("{:?}", notification);
        }
    });

    tokio::task::spawn(async move {
        loop {
            link_tx.publish("fuu/bar", "Hello World").unwrap();

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    broker.spawn(TokioSpawner).unwrap();

    handle.await.unwrap();
}

#[derive(Debug)]
struct TokioSpawner;

impl Spawner for TokioSpawner {
    fn spawn<S: Display + Send + 'static, F: Future<Output = ()> + Send + 'static>(
        &mut self,
        name: S,
        task: F,
    ) -> Result<(), Error> {
        tokio::spawn(async move {
            println!("Task started: {name}");

            task.await;

            println!("Task finished: {name}");
        });

        Ok(())
    }
}