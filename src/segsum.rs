#[cfg(test)]
use rand::distributions::{Distribution, Uniform};
#[cfg(test)]
use rand::thread_rng;

use std::cmp::*;
#[cfg(test)]
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
enum Change<T>
where
    T: Add<Output = T> + Mul<i64, Output = T> + Default + Debug + Clone,
{
    Abs(T),
    Rel(T),
}

#[derive(Debug, Clone)]
struct N<T>
where
    T: Add<Output = T> + Mul<i64, Output = T> + Default + Debug + Clone,
{
    l: i64,
    r: i64,
    s: T,
    left: Option<Box<N<T>>>,
    right: Option<Box<N<T>>>,
    mark: Option<Change<T>>, //lazy
}

impl<T> N<T>
where
    T: Add<Output = T> + Mul<i64, Output = T> + Default + Debug + Clone,
{
    pub fn new(l: i64, r: i64) -> Self {
        Self {
            l: l,
            r: r,
            s: Default::default(),
            left: None,
            right: None,
            mark: None,
        }
    }
    fn set_lazy_abs(&mut self, val: T) {
        self.mark = Some(Change::Abs(val.clone()));
        self.s = val;
    }
    fn set_lazy_rel(&mut self, val: T) {
        match &self.mark {
            Some(Change::Abs(_)) => {
                self.push_down_abs();
                assert!(self.mark.is_none());
            }
            _ => {}
        }
        match &mut self.mark {
            Some(Change::Rel(ref mut v)) => {
                *v = v.clone() + val;
            }
            _ => {
                self.mark = Some(Change::Rel(val));
            }
        }
    }
    pub fn range_sum(&self) -> T {
        match &self.mark {
            Some(Change::Abs(v)) => v.clone() * (self.r - self.l),
            Some(Change::Rel(v)) => self.s.clone() + v.clone() * (self.r - self.l),
            _ => self.s.clone(),
        }
    }
    fn extend(&mut self) {
        if self.l + 1 >= self.r {
            return;
        }
        self.extend_left();
        self.extend_right();
    }
    fn extend_left(&mut self) {
        let m = (self.l + self.r) / 2;
        if self.l < self.r && self.l < m {
            match self.left {
                None => {
                    self.left = Some(Box::new(N::new(self.l, m)));
                }
                _ => {}
            }
        }
    }
    fn extend_right(&mut self) {
        let m = (self.l + self.r) / 2;
        if self.l < self.r && m < self.r {
            match self.right {
                None => {
                    self.right = Some(Box::new(N::new(m, self.r)));
                }
                _ => {}
            }
        }
    }
    fn push_down_abs(&mut self) {
        use std::mem::swap;
        match &mut self.mark {
            Some(Change::Abs(ref mut a)) => {
                let mut x = Default::default();
                swap(&mut x, a);
                self.mark = None;
                self.extend();
                if let Some(ref mut y) = &mut self.left {
                    y.set_lazy_abs(x.clone());
                }
                if let Some(ref mut y) = &mut self.right {
                    y.set_lazy_abs(x.clone());
                }
                self.s = x * (self.r - self.l);
            }
            _ => {}
        }
    }
    fn push_down_rel(&mut self) {
        use std::mem::swap;
        match &mut self.mark {
            Some(Change::Rel(ref mut a)) => {
                let mut x = Default::default();
                swap(&mut x, a);
                self.mark = None;
                self.extend();
                if let Some(ref mut y) = &mut self.left {
                    y.set_lazy_rel(x.clone());
                }
                if let Some(ref mut y) = &mut self.right {
                    y.set_lazy_rel(x.clone());
                }
                self.s = self.s.clone() + x * (self.r - self.l);
            }
            _ => {}
        }
    }
    fn push_down(&mut self) {
        if let Some(ref x) = &self.mark {
            match x {
                Change::Abs(_) => {
                    self.push_down_abs();
                }
                _ => {
                    self.push_down_rel();
                }
            }
        }
    }
    pub fn update(&mut self, ll: i64, rr: i64, val: &T) {
        if ll <= self.l && self.r <= rr {
            self.set_lazy_abs(val.clone());
        } else {
            self.push_down();
            let m = (self.l + self.r) / 2;
            if ll < m {
                self.extend_left();
                match &mut self.left {
                    Some(ref mut x) => {
                        x.update(ll, min(m, rr), val);
                    }
                    _ => {}
                }
            }
            if m < rr {
                self.extend_right();
                match &mut self.right {
                    Some(ref mut x) => {
                        x.update(max(m, ll), rr, val);
                    }
                    _ => {}
                }
            }
            self.s = self
                .left
                .as_mut()
                .map_or(Default::default(), |x| x.range_sum())
                + self
                    .right
                    .as_mut()
                    .map_or(Default::default(), |x| x.range_sum());
            debug_assert!(self.mark.is_none());
        }
    }
    pub fn add(&mut self, ll: i64, rr: i64, delta: &T) {
        if ll <= self.l && self.r <= rr {
            self.set_lazy_rel(delta.clone());
        } else {
            self.push_down();
            let m = (self.l + self.r) / 2;
            if ll < m {
                self.extend();
                if let Some(ref mut x) = &mut self.left {
                    x.add(ll, min(m, rr), delta);
                }
            }
            if m < rr {
                self.extend();
                if let Some(ref mut x) = &mut self.right {
                    x.add(max(m, ll), rr, delta);
                }
            }
            self.s = self
                .left
                .as_mut()
                .map_or(Default::default(), |x| x.range_sum())
                + self
                    .right
                    .as_mut()
                    .map_or(Default::default(), |x| x.range_sum());
            debug_assert!(self.mark.is_none());
        }
    }
    pub fn query_sum(&mut self, ll: i64, rr: i64) -> T {
        if ll >= rr || max(ll, self.l) >= min(rr, self.r) {
            Default::default()
        } else if ll <= self.l && self.r <= rr {
            self.range_sum()
        } else {
            self.push_down();
            let mut ret: T = Default::default();
            let m = (self.l + self.r) / 2;
            if ll < m {
                if let Some(ref mut x) = &mut self.left {
                    ret = ret + x.query_sum(ll, min(m, rr));
                }
            }
            if m < rr {
                if let Some(ref mut x) = &mut self.right {
                    ret = ret + x.query_sum(max(m, ll), rr);
                }
            }
            ret
        }
    }
    #[cfg(test)]
    pub fn dbg(&self, d_parent: i64, depths: &mut HashMap<i64, i64>) -> usize {
        *depths.entry(d_parent).or_default() += 1;
        let mut count = 0;
        if let Some(ref x) = &self.left {
            count += x.dbg(d_parent + 1, depths);
        }
        println!(
            "[{},{}), mark: {:?}, s: {:?}",
            self.l, self.r, self.mark, self.s
        );
        if let Some(ref x) = &self.right {
            count += x.dbg(d_parent + 1, depths);
        }
        count + 1
    }
}

#[derive(Debug, Clone)]
pub struct SegSum<T>
where
    T: Add<Output = T> + Mul<i64, Output = T> + Default + Debug + Clone,
{
    lim_l: i64,
    lim_r: i64,
    r: N<T>,
}

impl<T> SegSum<T>
where
    T: Add<Output = T> + Mul<i64, Output = T> + Default + Debug + Clone,
{
    pub fn new(l: i64, r: i64) -> Self {
        Self {
            lim_l: l,
            lim_r: r,
            r: N::new(l, r),
        }
    }
    pub fn add(&mut self, l: i64, r: i64, delta: &T) {
        if l < r {
            assert!(self.lim_l <= l);
            assert!(r <= self.lim_r);
            self.r.add(l, r, delta);
        }
    }
    pub fn query_range(&mut self, l: i64, r: i64) -> T {
        assert!(self.lim_l <= l);
        assert!(r <= self.lim_r);
        self.r.query_sum(l, r)
    }
    pub fn update(&mut self, l: i64, r: i64, val: &T) {
        if l < r {
            assert!(self.lim_l <= l);
            assert!(r <= self.lim_r);
            self.r.update(l, r, val);
        }
    }
    #[cfg(test)]
    pub fn dbg(&self) {
        let mut count = 0;
        let mut depths = HashMap::new();
        self.r.dbg(0, &mut depths);
        let mut d_avg = 0.;
        for (depth, freq) in depths {
            println!("depth: {}, count: {}", depth, freq);
            d_avg += (depth * freq) as f64;
            count += freq;
        }
        d_avg /= count as f64;
        println!("avg depth per node: {}", d_avg);
        println!("count nodes: {}", count);
    }
}

#[test]
fn test_seg() {
    const M: usize = 64;
    let mut seg = SegSum::new(0, M as i64);
    let mut reference = vec![0; M];
    let mut g = thread_rng();
    let distr = Uniform::from(0..M as i64);
    let distr2 = Uniform::from(0..M as i64 + 1);
    let distr3 = Uniform::from(-10..11);
    for _ in 0..10000 {
        if distr.sample(&mut g) % 2 == 0 {
            let mut a: i64 = distr.sample(&mut g);
            let mut b: i64 = max(distr2.sample(&mut g), a);
            while a >= b {
                a = distr.sample(&mut g);
                b = max(distr2.sample(&mut g), a);
            }
            let delta: i64 = distr3.sample(&mut g);
            println!("add: [{},{}), {}", a, b, delta);
            seg.add(a as _, b as _, &delta);
            for j in a..b {
                reference[j as usize] += delta;
            }
        } else {
            let mut a: i64 = distr.sample(&mut g);
            let mut b: i64 = max(distr2.sample(&mut g), a);
            while a >= b {
                a = distr.sample(&mut g);
                b = max(distr2.sample(&mut g), a);
            }
            let val: i64 = distr3.sample(&mut g);
            seg.update(a as i64, b as i64, &val);
            for h in a..b {
                reference[h as usize] = val;
            }
            println!("update: [{},{}), {}", a, b, val);
        }
    }

    for i in 1..M {
        for j in 1..M + 1 {
            let v = seg.query_range(i as i64, j as i64);
            let mut expect = 0;
            for k in i..j {
                expect += reference[k];
            }
            assert_eq!(expect, v, "not matching: [{},{}), {}, {}", i, j, expect, v);
        }
    }
}
