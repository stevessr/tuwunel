use criterion::{Criterion, criterion_group, criterion_main};
use tuwunel::Server;
use tuwunel_core::{Args, runtime};

criterion_group!(
	name = benches;
	config = Criterion::default().sample_size(10);
	targets = dummy, smoke
);

criterion_main!(benches);

fn dummy(c: &mut Criterion) { c.bench_function("dummy", |c| c.iter(|| {})); }

fn smoke(c: &mut Criterion) {
	c.bench_function("smoke", |c| {
		c.iter(|| {
			let args = Args::default_test(&["smoke", "cleanup"]);
			let runtime = runtime::new(Some(&args))?;
			let server = Server::new(Some(&args), Some(runtime.handle()))?;
			tuwunel::exec(&server, runtime)
		});
	});
}
