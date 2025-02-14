use crate::Matrix;
use dwconsts::*;
use itertools::multizip;
use lapack_sys::*;
use num_traits::Zero;
use std::ops::AddAssign;
use std::ops::Mul;
use types::c64;

impl Matrix<c64> {
    pub fn identity(n: usize) -> Matrix<c64> {
        let mut mat = Matrix::<c64>::new(n, n);

        for v in mat.as_mut_slice() {
            *v = c64::zero();
        }

        for i in 0..n {
            mat[[i, i]] = ONE_C64;
        }

        mat
    }

    pub fn adjoint(&self) -> Matrix<c64> {
        let mut data = Vec::with_capacity(self.nrow * self.ncol);
        for i in 0..self.nrow {
            for j in 0..self.ncol {
                data.push(self[[i, j]].clone().conj())
            }
        }
        Matrix {
            nrow: self.ncol,
            ncol: self.nrow,
            data,
        }
    }

    pub fn sum(&self) -> c64 {
        return self.data.iter().sum();
    }

    pub fn action(&self, vin: &[c64], vout: &mut [c64]) {
        vout.iter_mut().for_each(|x| *x = c64::zero());

        for i in 0..self.ncol {
            for j in 0..self.nrow {
                vout[j] += self[[j, i]] * vin[i];
            }
        }
    }

    // pub fn dot(&self, v: &[c64]) -> Vec<c64> {
    //     let mut vout = vec![ZERO_C64; self.nrow];

    //     for i in 0..self.ncol {
    //         for j in 0..self.nrow {
    //             vout[j] += self[[j, i]] * v[i];
    //         }
    //     }

    //     vout
    // }

    pub fn inv(&mut self) {
        //        self.pinv();

        //return;

        let nn = self.nrow;
        let n = nn as i32;

        let mut ipiv = vec![0i32; nn];
        let lwork = n * n;
        let mut work = vec![c64::zero(); lwork as usize];
        let mut info = 0i32;

        unsafe {
            zgetrf_(&n, &n, self.as_ptr(), &n, ipiv.as_mut_ptr(), &mut info);
            zgetri_(
                &n,
                self.as_mut_ptr(),
                &n,
                ipiv.as_ptr(),
                work.as_mut_ptr(),
                &lwork,
                &mut info,
            );
        }
    }

    pub fn pinv(&mut self) {
        let n = self.nrow as i32;
        let nrhs = n;

        let mut s = vec![0.0; n as usize];
        let rcond: f64 = EPS30;
        let lwork = n + 2 * n; //2*min(M,N) + max(M,N,NRHS)
        let mut work = vec![c64::zero(); lwork as usize];
        let mut rwork = vec![0.0f64; (5 * n) as usize];
        let mut info = 0i32;

        let mut b = Matrix::<c64>::identity(n as usize);

        unsafe {
            zgelss_( // encut >= 700 eV 时报错。
                &n,
                &n,
                &nrhs,
                self.as_mut_ptr(),
                &n,
                b.as_mut_ptr(),
                &n,
                s.as_mut_ptr(),
                &rcond,
                &n,
                work.as_mut_ptr(),
                &lwork,
                rwork.as_mut_ptr(),
                &mut info,
            );
        }

        self.data.copy_from_slice(b.as_slice());
    }

    pub fn mat_mul(&self, rhs: &Matrix<c64>) -> Matrix<c64> {
        let n = self.nrow;
        let mut mat = Matrix::<c64>::new(n, n);

        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    mat[[i, j]] += self[[i, k]] * rhs[[k, j]];
                }
            }
        }

        mat
    }
}

impl Mul<f64> for Matrix<c64> {
    type Output = Matrix<c64>;

    fn mul(self, rhs: f64) -> Matrix<c64> {
        let mut mat = self.clone();

        for v in mat.data.iter_mut() {
            *v *= rhs;
        }

        mat
    }
}

impl AddAssign<Matrix<f64>> for Matrix<c64> {
    fn add_assign(&mut self, rhs: Matrix<f64>) {
        for (s, d) in multizip((rhs.as_slice().iter(), self.as_mut_slice().iter_mut())) {
            *d += *s;
        }
    }
}
