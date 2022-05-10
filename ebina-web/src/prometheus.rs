use ebina_types::*;

use tide::Request;
use prometheus::{Opts, Registry, TextEncoder, Encoder, IntGauge, IntCounterVec};
use crate::State;

pub async fn prometheus_metrics(req: Request<State>) -> tide::Result {
	let state= req.state();
	let ctx = &state.ctx;
	let data = ctx.data.read().await;
	let commands = data.get::<CommandCounter>().unwrap();

	// Create a Counter.
	let counter_opts = Opts::new("ebina_server_count", "Number of servers bot is in");
	let gauge = IntGauge::with_opts(counter_opts)?;

	// Create counter for commands
	let command_counter_opts = Opts::new("ebina_command_count", "Number of commands issued");
	let command_counter = IntCounterVec::new(command_counter_opts, &["command"])?;

	commands.iter().for_each(|(command, count)| {
		command_counter.with_label_values(&[command]).inc_by(*count);
	});

	// Create a UrlCounter.
	let url_counter_opts = Opts::new("ebina_url_count", "Number of urls handled");
	let url_counter = IntCounterVec::new(url_counter_opts, &["domain"])?;

	let urls = data.get::<UrlCounter>().unwrap();

	urls.iter().for_each(|(url, count)| {
		url_counter.with_label_values(&[url]).inc_by(*count);
	});

	// Create a Registry and register Counter.
	let r = Registry::new();
	r.register(Box::new(gauge.clone()))?;
	r.register(Box::new(command_counter))?;
	r.register(Box::new(url_counter))?;


	gauge.set(ctx.cache.guild_count().try_into().unwrap());

	// Gather the metrics.
	let mut buffer = vec![];
	let encoder = TextEncoder::new();
	let metric_families = r.gather();
	encoder.encode(&metric_families, &mut buffer).unwrap();

	Ok(String::from_utf8(buffer).unwrap().into())
}