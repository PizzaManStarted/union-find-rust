use crate::union_find::{Sites, UnionFindAlgo};

#[derive(Debug)]
pub struct QuickUnion {
    sites: Sites,
}

impl UnionFindAlgo for QuickUnion {
    fn new(n: usize) -> Self {
        Self {
            sites: Sites::new(n),
        }
    }

    fn find(&mut self, mut p: usize) -> usize {
        while p != self.sites.get(p) {
            p = self.sites.get(p);
        }
        p
    }

    fn union(&mut self, p: usize, q: usize) {
        let p_root = self.find(p);
        let q_root = self.find(q);

        if p_root == q_root {
            return;
        }

        self.sites.set(p_root, q_root);

        self.sites.count -= 1;
    }

    fn get_sites(&self) -> &Sites {
        &self.sites
    }
}

#[derive(Debug)]
pub struct QuickFind {
    sites: Sites,
}

impl UnionFindAlgo for QuickFind {
    fn new(n: usize) -> Self {
        Self {
            sites: Sites::new(n),
        }
    }

    fn find(&mut self, p: usize) -> usize {
        self.sites.get(p)
    }

    fn union(&mut self, p: usize, q: usize) {
        let p_id = self.find(p);
        let q_id = self.find(q);

        if p_id == q_id {
            return;
        }

        for i in 0..self.sites.length() {
            if self.sites.get(i) == p_id {
                self.sites.set(i, q_id);
            }
        }
        self.sites.count -= 1;
    }

    fn get_sites(&self) -> &Sites {
        &self.sites
    }
}
