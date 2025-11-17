// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::Result;
use std::collections::HashMap;

const PHI: f64 = 1.618033988749895; // Exact φ = (1 + √5)/2
const PI: f64 = std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct NodeEmbed {
    pub pos: Vec<f32>, // [x, y] disk
}

pub fn poincare_embed(deps: &HashMap<String, Vec<String>>) -> Result<Vec<NodeEmbed>> {
    let mut embeds = Vec::new();

    // Root at center
    embeds.push(NodeEmbed {
        pos: vec![0.0, 0.0],
    });

    // Assume root "root", children deps["root"]
    if let Some(children) = deps.get("root") {
        let len = children.len() as f64;
        let d = len * PHI / 2.0; // Scale
        let r_base = d.tanh(); // Curvature -1 disk
        for (i, child) in children.iter().enumerate() {
            let theta = 2.0 * PI * i as f64 / len;
            let r = r_base / (1.0 + i as f64 * 0.1); // Inner closer
            let x = (r * theta.cos()) as f32;
            let y = (r * theta.sin()) as f32;
            // Ensure r <1
            let norm = (x.powi(2) + y.powi(2)) as f64;
            if norm >= 1.0 {
                let scale = 0.99 / norm.sqrt();
                embeds.push(NodeEmbed {
                    pos: vec![(x as f64 * scale) as f32, (y as f64 * scale) as f32],
                });
            } else {
                embeds.push(NodeEmbed { pos: vec![x, y] });
            }
            // Recurse for sub-deps if tree
        }
    }

    Ok(embeds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poincare_embed() {
        let mut deps = HashMap::new();
        deps.insert(
            "root".to_string(),
            vec!["child1".to_string(), "child2".to_string()],
        );
        let embeds = poincare_embed(&deps).unwrap();
        assert_eq!(embeds.len(), 3);
        for embed in &embeds[1..] {
            let r_sq = embed.pos[0].powi(2) + embed.pos[1].powi(2);
            assert!(r_sq < 1.0);
        }
        // Theta uniform approx
        let thetas: Vec<f64> = embeds[1..]
            .iter()
            .map(|e| (e.pos[1] / e.pos[0]).atan())
            .collect();
        assert!((thetas[0] - 0.0).abs() < 1.0); // Rough
        assert!((thetas[1] - PI).abs() < 1.0);
    }
}
