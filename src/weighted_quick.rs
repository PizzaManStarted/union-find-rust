use crate::union_find::{Sites, UnionFindAlgo};

pub struct WeightedQuickUnion {
    sites: Sites,
    lengths: Vec<usize>,
}

impl UnionFindAlgo for WeightedQuickUnion {
    fn new(n: usize) -> Self {
        Self {
            sites: Sites::new(n),
            lengths: vec![1; n],
        }
    }

    fn find(&mut self, mut p: usize) -> usize {
        while p != self.sites.arr[p] {
            p = self.sites.arr[p];
        }
        p
    }

    fn union(&mut self, p: usize, q: usize) {
        let i = self.find(p);
        let j = self.find(q);
        if i == j {
            return;
        }
        if self.lengths[i] < self.lengths[j] {
            self.sites.arr[i] = j;
            self.lengths[j] += self.lengths[i];
        } else {
            self.sites.arr[j] = i;
            self.lengths[i] += self.lengths[j];
        }
        self.sites.count -= 1;
    }

    fn get_sites(&self) -> &Sites {
        &self.sites
    }
}
