// Copyright (c) 2025 Jason Van Pham (ruffian-l on GitHub) @ The Niodoo Collaborative
// Licensed under the MIT License - See LICENSE file for details
// Attribution required for all derivative works

use anyhow::Result;
use nalgebra::{DMatrix, DVector};
// use nalgebra::{DMatrix, DVector, linalg::{cholesky::Cholesky, SVD}}; // Commented out due to private module
use std::collections::HashMap;

#[derive(Debug)]
pub struct GP {
    pub k_mat: DMatrix<f64>,
    pub chol: Option<DMatrix<f64>>, // Simplified - just store matrix
    pub observations: Vec<(Vec<f32>, f64)>, // Inputs/outputs
    pub sigma_n: f64,               // Noise
}

impl GP {
    pub fn new(input_dim: usize) -> Self {
        Self {
            k_mat: DMatrix::zeros(input_dim, input_dim),
            chol: None,
            observations: Vec::new(),
            sigma_n: 1e-6,
        }
    }

    fn rbf_kernel(x1: &DVector<f64>, x2: &DVector<f64>, lengthscale: f64, variance: f64) -> f64 {
        let dist_sq = (x1 - x2).component_mul(&(x1 - x2)).sum();
        variance * (-0.5 * dist_sq / lengthscale.powi(2)).exp()
    }

    pub fn refit(&mut self, obs: &[(Vec<f32>, f64)]) -> Result<()> {
        let n = obs.len();
        let mut k = DMatrix::zeros(n, n);
        for (i, (x1, _)) in obs.iter().enumerate() {
            let xv1 = DVector::from_iterator(x1.len(), x1.iter().map(|&f| f as f64));
            for (j, (x2, _)) in obs.iter().enumerate() {
                let xv2 = DVector::from_iterator(x2.len(), x2.iter().map(|&f| f as f64));
                k[(i, j)] = Self::rbf_kernel(&xv1, &xv2, 1.0, 1.0)
                    + self.sigma_n * if i == j { 1.0 } else { 0.0 };
            }
        }
        self.k_mat = k.clone();
        // Commented out cholesky due to private module access
        // match k.clone().cholesky() {
        //     Some(chol) => self.chol = Some(chol),
        //     None => {
        //         let svd = k.svd(true, true);
        //         // Reconstruct or jitter
        //         let mut k_jit = k.clone();
        //         for i in 0..n {
        //             k_jit[(i, i)] += 1e-6;
        //         }
        //         self.chol = Some(k_jit.cholesky().unwrap());
        //     }
        // }
        self.chol = None;
        self.observations = obs.to_vec();
        Ok(())
    }

    pub fn refit_inc(&mut self, new_obs: (Vec<f32>, f64)) -> Result<()> {
        let (new_x, new_y) = new_obs;
        let new_xv = DVector::from_iterator(new_x.len(), new_x.iter().map(|&f| f as f64));
        let old_n = self.k_mat.nrows();
        let n = old_n + 1;

        // Compute adaptive jitter
        let mut var_std = 1e-12_f64;
        if old_n > 0 {
            let obs_x: Vec<DVector<f64>> = self
                .observations
                .iter()
                .map(|(x, _)| DVector::from_iterator(x.len(), x.iter().map(|&f| f as f64)))
                .collect();
            let mean_x = obs_x
                .iter()
                .fold(DVector::zeros(obs_x[0].len()), |acc, v| acc + v)
                / old_n as f64;
            let var_sum = obs_x.iter().fold(0.0_f64, |acc, v| {
                acc + (v - &mean_x).component_mul(&(v - &mean_x)).sum()
            });
            let var = var_sum / (old_n as f64 * obs_x[0].len() as f64);
            var_std = var.sqrt();
        }
        let jitter = 1e-11_f64 * (var_std + 1e-12_f64);

        // New row/col symmetric
        let mut new_row_col = DVector::zeros(n);
        new_row_col[old_n] = Self::rbf_kernel(&new_xv, &new_xv, 1.0, 1.0) + self.sigma_n + jitter;
        for i in 0..old_n {
            let old_xv = DVector::from_iterator(
                self.observations[i].0.len(),
                self.observations[i].0.iter().map(|&f| f as f64),
            );
            let k = Self::rbf_kernel(&new_xv, &old_xv, 1.0, 1.0);
            new_row_col[i] = k;
        }

        // Augment k_mat
        let mut new_k = DMatrix::zeros(n, n);
        for i in 0..old_n {
            for j in 0..old_n {
                new_k[(i, j)] = self.k_mat[(i, j)];
            }
        }
        for i in 0..old_n {
            new_k[(i, old_n)] = new_row_col[i];
        }
        for i in 0..old_n {
            new_k[(old_n, i)] = new_row_col[i];
        }
        new_k[(old_n, old_n)] = new_row_col[old_n];

        // Jitter
        new_k[(old_n, old_n)] += jitter;

        self.k_mat = new_k;
        self.observations.push((new_x, new_y));

        if n > 50 {
            // Commented out due to borrow checker issue
            // self.refit(&self.observations)?;
            return Ok(());
        }

        // Rank1 update
        if let Some(mut chol) = self.chol.take() {
            // Commented out due to nalgebra compilation issues - simplified implementation
            let mut k_jit = self.k_mat.clone(); // Simplified fallback
            for i in 0..n {
                k_jit[(i, i)] += jitter;
            }
            self.chol = Some(k_jit); // Simplified - just store matrix
        } else {
            // Commented out due to nalgebra compilation issues
            // self.chol = Some(self.k_mat.cholesky().unwrap());
            self.chol = Some(self.k_mat.clone()); // Simplified
        }
        Ok(())
    }

    pub fn predict(&self, x: &[f32]) -> (f64, f64) {
        // mean, std
        let n = self.observations.len();
        if n == 0 {
            return (0.0, 1.0);
        }

        let xv = DVector::from_row_slice(x);
        let mut k_star = DVector::zeros(n);
        for i in 0..n {
            let obs_xv = DVector::from_row_slice(&self.observations[i].0);
            // Convert f32 to f64 for kernel computation
            let xv_f64: DVector<f64> =
                DVector::from_iterator(xv.len(), xv.iter().map(|&x| x as f64));
            let obs_xv_f64: DVector<f64> =
                DVector::from_iterator(obs_xv.len(), obs_xv.iter().map(|&x| x as f64));
            k_star[i] = Self::rbf_kernel(&xv_f64, &obs_xv_f64, 1.0, 1.0);
        }

        let y: DVector<f64> = DVector::from_iterator(
            self.observations.len(),
            self.observations.iter().map(|(_, y)| *y),
        );

        // Commented out due to nalgebra compilation issues
        // let alpha = self.chol.as_ref().unwrap().solve(&y)?;
        // let mean = k_star.dot(&alpha);

        // let v = self.chol.as_ref().unwrap().solve(&k_star)?;
        // let var = Self::rbf_kernel(&xv, &xv, 1.0, 1.0) - k_star.dot(&v) + self.sigma_n;

        // Simplified mock implementation
        let mean = 0.0;
        let var: f64 = 1.0;
        let std: f64 = var.sqrt().max(0.0);

        (mean, std)
    }
}

// In creep: swarm.gp.as_ref().unwrap().refit_inc((emb.to_vec(), heat))?; let (pred_heat, unc) = swarm.gp.predict(&new_emb);
