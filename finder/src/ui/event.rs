/*
 * File from https://github.com/fdehau/tui-rs/blob/master/examples/util/event.rs
 * Copyright (c) 2016 Florian Dehau
 * Distributed under MIT license - https://github.com/fdehau/tui-rs/blob/master/LICENSE
 */

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event<I> {
	Input(I),
	Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
	rx: mpsc::Receiver<Event<Key>>,
	#[allow(dead_code)]
	input_handle: thread::JoinHandle<()>,
	#[allow(dead_code)]
	tick_handle: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
	pub exit_key: Key,
	pub tick_rate: Duration,
}

impl Default for Config {
	fn default() -> Config {
		Config {
			exit_key: Key::Char('q'),
			tick_rate: Duration::from_millis(250),
		}
	}
}

impl Events {
	pub fn new() -> Events {
		Events::with_config(Config::default())
	}

	pub fn with_config(config: Config) -> Events {
		let (tx, rx) = mpsc::channel();
		let input_handle = {
			let tx = tx.clone();
			thread::spawn(move || {
				let stdin = io::stdin();
				for key in stdin.keys().flatten() {
					if tx.send(Event::Input(key)).is_err() {
						return;
					}
					if key == config.exit_key {
						std::process::exit(0);
					}
				}
			})
		};
		let tick_handle = {
			thread::spawn(move || {
				let tx = tx.clone();
				loop {
					tx.send(Event::Tick).unwrap();
					thread::sleep(config.tick_rate);
				}
			})
		};
		Events {
			rx,
			input_handle,
			tick_handle,
		}
	}

	pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
		self.rx.recv()
	}
}
