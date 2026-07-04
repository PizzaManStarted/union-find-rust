use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::PathBuf,
};
use strum_macros::EnumIter;

use crate::{
    quick::{QuickFind, QuickUnion},
    weighted_quick::WeightedQuickUnion,
};

#[derive(Debug, EnumIter, Default, PartialEq, Eq, Clone)]
pub enum UnionFindChoice {
    #[default]
    QuickFind,
    QuickUnion,
    WeightedQuickUnion,
}

impl UnionFindChoice {
    pub fn create(&self, n: usize) -> Box<dyn UnionFindAlgo> {
        match self {
            UnionFindChoice::QuickFind => Box::new(QuickFind::new(n)),
            UnionFindChoice::QuickUnion => Box::new(QuickUnion::new(n)),
            UnionFindChoice::WeightedQuickUnion => Box::new(WeightedQuickUnion::new(n)),
        }
    }
}

// TODO: To make things easier with the enum choice: the algo should have access to everything and cannot have a parameter when created ? good luck
pub trait UnionFindAlgo {
    fn new(n: usize) -> Self
    where
        Self: Sized;

    fn find(&mut self, p: usize) -> usize;
    fn union(&mut self, p: usize, q: usize);

    fn connected(&mut self, p: usize, q: usize) -> bool {
        self.find(p) == self.find(q)
    }

    fn get_sites(&self) -> &Sites;
}

#[derive(Debug, Clone)]
pub struct Sites {
    pub arr: Vec<usize>,
    pub count: usize,
}

impl Sites {
    pub fn new(n: usize) -> Self {
        Self {
            arr: (0..n).collect(),
            count: n,
        }
    }

    pub fn length(&self) -> usize {
        self.arr.len()
    }
}

pub struct UnionFind {
    lines: Lines<BufReader<File>>,
    algo: Box<dyn UnionFindAlgo>,
    next: Option<(usize, usize)>,
    n: usize,
}

impl UnionFind {
    pub fn new(file_name: &PathBuf, choice: &UnionFindChoice) -> io::Result<Self> {
        let file = File::open(file_name)?;

        let file_reader = BufReader::new(file);

        let mut lines: io::Lines<BufReader<File>> = file_reader.lines();

        let n = lines
            .next()
            .expect("First value present")?
            .parse::<usize>()
            .expect("correct n value");

        let algo = choice.create(n);

        let next = match lines.next() {
            Some(val) => read_usize_pair(val.expect("Correct val")),
            None => None,
        };

        Ok(Self {
            lines,
            algo,
            next,
            n,
        })
    }

    pub fn get_n(&self) -> usize {
        self.n
    }

    pub fn to_tree(&self) -> (Vec<usize>, Vec<Vec<usize>>) {
        let mut children = vec![Vec::new(); self.n];
        let mut roots = Vec::new();

        for (i, &parent) in self.algo.get_sites().arr.iter().enumerate() {
            if i == parent {
                roots.push(i);
            } else {
                children[parent].push(i);
            }
        }

        (roots, children)
    }

    pub fn get_sites(&self) -> &Vec<usize> {
        &self.algo.get_sites().arr
    }

    pub fn peak_next(&self) -> Option<(usize, usize)> {
        self.next
    }
}

impl Iterator for UnionFind {
    type Item = (usize, usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let (p, q) = self.next?;

        let res = if self.algo.connected(p, q) {
            Some((p, q, false))
        } else {
            self.algo.union(p, q);
            Some((p, q, true))
        };

        self.next = match self.lines.next() {
            Some(val) => read_usize_pair(val.expect("Correct val")),
            None => None,
        };

        res
    }
}

// pub fn read_file<U: UnionFindAlgo>(file_name: impl ToString) -> io::Result<()> {
//     let file = File::open(file_name.to_string())?;
//     let reader = BufReader::new(file);

//     let mut lines = reader.lines();

//     let n = lines
//         .next()
//         .expect("First value present")?
//         .parse::<usize>()
//         .expect("correct n value");

//     let mut uf = U::new(n);
//     let mut u = 0;

//     for line in lines.map_while(Result::ok) {
//         let (p, q) = read_usize_pair(line).expect("line not empty");
//         u += 1;

//         if uf.connected(p, q) {
//             continue;
//         }

//         uf.union(p, q);

//         println!("{p} - {q}");
//     }
//     println!("{u}");

//     Ok(())
// }

fn read_usize_pair(line: String) -> Option<(usize, usize)> {
    let mut values = line.split(" ");

    Some((
        values.next()?.parse::<usize>().expect("correct p value"),
        values.next()?.parse::<usize>().expect("correct q value"),
    ))
}
