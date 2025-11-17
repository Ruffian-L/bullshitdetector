// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use std::sync::{Arc, RwLock};
use tracing::info;

pub struct OverEngineeredSample {
    data: Arc<RwLock<Vec<i32>>>,
}

impl OverEngineeredSample {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn add(&self, val: i32) {
        let mut d = self.data.write().unwrap();
        d.push(val);
    }

    pub fn get_first(&self) -> i32 {
        self.data.read().unwrap()[0]
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let s = OverEngineeredSample::new();
    s.add(42);
    info!("First: {}", s.get_first());
}
