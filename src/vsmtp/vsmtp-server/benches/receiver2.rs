use criterion::{criterion_group, criterion_main, Criterion};
use vsmtp_config::DnsResolvers;
use vsmtp_rule_engine::RuleEngine;
use vsmtp_server::{socket_bind_anyhow, Server};
use vsmtp_test::config;

fn get_mail(body_size: u64) -> lettre::Message {
    lettre::Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body((0..body_size).map(|_| 'x').collect::<String>())
        .unwrap()
}

fn run_benchmark(body_size: u64, port: u16) {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let server = tokio::spawn(async move {
                let config = config::local_test();

                let (emitter, _, _) = vsmtp_server::scheduler::init(1, 1);

                let config = std::sync::Arc::new(config);
                let resolvers = std::sync::Arc::new(DnsResolvers::from_system_conf().unwrap());

                let queue_manager =
                    <vqueue::temp::QueueManager as vqueue::GenericQueueManager>::init(
                        config.clone(),
                        vec![],
                    )
                    .unwrap();

                let rule_engine = std::sync::Arc::new(
                    RuleEngine::new(config.clone(), resolvers.clone(), queue_manager).unwrap(),
                );

                Server::new(
                    config.clone(),
                    rule_engine.clone(),
                    <vqueue::temp::QueueManager as vqueue::GenericQueueManager>::init(
                        config.clone(),
                        vec![],
                    )
                    .unwrap(),
                    emitter,
                )
                .unwrap()
                .listen((
                    [format!("127.0.0.1:{port}")]
                        .iter()
                        .map(socket_bind_anyhow)
                        .collect::<anyhow::Result<Vec<std::net::TcpListener>>>()
                        .unwrap(),
                    vec![],
                    vec![],
                ))
                .await
                .unwrap();
            });

            let client = tokio::spawn(async move {
                let sender =
                    lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(
                        "localhost".to_string(),
                    )
                    .port(port)
                    .build();

                lettre::AsyncTransport::send(&sender, get_mail(body_size)).await
            });

            tokio::select! {
                biased;
                server = server => {
                    let mut file = std::fs::File::create("./tmp/server.txt").unwrap();
                    std::io::Write::write_all(&mut file, format!("{server:?}").as_bytes()).unwrap();
                    server.unwrap();
                },
                client = client => {
                    let mut file = std::fs::File::create("./tmp/client.txt").unwrap();
                    std::io::Write::write_all(&mut file, format!("{client:?}").as_bytes()).unwrap();
                    client.unwrap().unwrap();
                },
            }
        });
}

fn criterion_receiver_1024(c: &mut Criterion) {
    c.bench_function("iai_receiver_1024", |b| {
        b.iter(|| run_benchmark(1024, 12000 + rand::random::<u16>().rem_euclid(10000)))
    });
}

fn criterion_receiver_1048576(c: &mut Criterion) {
    c.bench_function("iai_receiver_1048576", |b| {
        b.iter(|| run_benchmark(1048576, 12000 + rand::random::<u16>().rem_euclid(10000)))
    });
}

criterion_group!(benches, criterion_receiver_1024, criterion_receiver_1048576);
criterion_main!(benches);
