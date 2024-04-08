pub mod mat {
    use std::fmt;

    pub struct Element<T> {
        idx : usize,
        data : T
    }

    pub type SparseVector<T> = Vec<Element<T>>;

    pub struct SparseMatrix<T > where T: std::cmp::PartialEq
    {
        cols : usize,
        rows : usize,
        def : T,
        data : SparseVector<SparseVector<T>>
    }

    impl<T : std::cmp::PartialEq> SparseMatrix<T> {
        pub fn len(&self) -> usize {
            return self.data.len();
        }

        pub fn actual_size(&self) -> usize {
            self.data.iter().map(|x| x.data.len()).sum()
        }

        pub fn new(cols : usize, rows : usize, def : T) -> SparseMatrix<T> {
            return SparseMatrix {
                cols,
                rows,
                def,
                data: vec![]
            }
        }

        pub fn rows(&self) -> usize {self.rows}
        pub fn cols(&self) -> usize {self.cols}
    }

    impl<T : std::cmp::PartialEq> std::ops::Index<[usize;2]> for SparseMatrix<T> {
        type Output = T;
        fn index(&self, idx : [usize;2]) -> &T {
            let (re, ri) = svec_find_idx(&self.data, idx[1]);
            if !re {
                return &self.def;
            }

            let (ce, ci) = svec_find_idx(&self.data[ri].data, idx[0]);
            if !ce {
                return &self.def;
            }

            return &self.data[ri].data[ci].data;
        }
    }

    fn svec_find_idx<T>(svec : &SparseVector<T>, cidx : usize) -> (bool, usize)
    {
        match svec.binary_search_by(|elem| { elem.idx.cmp(&cidx) }) {
            Ok(pos) => { return (true, pos); }
            Err(pos) => { return (false, pos); },
        };
    }

    impl <T : std::fmt::Display + std::cmp::PartialEq> SparseMatrix<T> {
        pub fn insert(&mut self, cidx : usize, ridx : usize, val : T )
        {
            if ridx >= self.rows() || cidx >= self.cols() {
                panic!("Matrix too small.\n\tInserting ({cidx}, {ridx}) into matrix of size {}x{}.", self.cols(), self.rows());
            }

            let (re, ri) = svec_find_idx(&self.data, ridx);
            if !re && val != self.def {
                self.data.insert(ri, Element {
                    idx : ridx,
                    data :vec![Element { idx: cidx, data : val}]
                });

                return;
            }

            let (ce, ci) = svec_find_idx(&self.data[ri].data, cidx);

            if ce {
                if val == self.def {
                    self.data[ri].data.remove(ci);
                    if self.data[ri].data.len() == 0 {
                        self.data.remove(ri);
                    }
                    return;
                }

                self.data[ri].data[ci].data = val;
                return;
            }

            if !ce && val != self.def {
                self.data[ri].data.insert(ci, Element {
                    idx : cidx,
                    data : val
                });
            }
        }

        pub fn random_default(&self) -> Option<(usize, usize)>{
            let sa = self.actual_size();
            let total = self.rows() * self.cols() - self.actual_size();
            println!("total={total}, sa={sa}");
            if total == 0 {
                return None;
            }

            let rand_idx = rand::Rng::gen_range(&mut rand::thread_rng(), 0..total);
            println!("Default index {} from {}", rand_idx, total);

            let mut cridx = 0;
            let mut ccidx = 0;

            let mut num_empty = 0;
            let mut nnum_empty;

            for row_elem in &self.data {
                nnum_empty = num_empty + (row_elem.idx - cridx) * self.cols();
                let new_row_idx = row_elem.idx;
                println!("rand_idx={rand_idx}, num_empty={num_empty}, nnum_empty={nnum_empty}, cridx={cridx},  new_row_idx={new_row_idx}:");
                if  nnum_empty > rand_idx {
                    let pos = cridx * self.cols();
                    let npos = (rand_idx - num_empty) + pos;
                    let r = npos / self.cols();
                    let c = npos % self.cols();
                    return Some((c, r));
                }

                num_empty = nnum_empty;
                cridx = row_elem.idx;
                ccidx = 0;

                for elem in &row_elem.data {
                    nnum_empty = num_empty + (elem.idx - ccidx);
                    if  nnum_empty > rand_idx {
                        let c = rand_idx - num_empty + ccidx;
                        return Some((c, cridx));
                    }
                    num_empty = nnum_empty;
                    ccidx = elem.idx + 1;
                }

                nnum_empty = num_empty + (self.cols() - ccidx);

                if nnum_empty > rand_idx {
                    let c = rand_idx - num_empty + ccidx;
                    return Some((c, cridx));
                }

                num_empty = nnum_empty;
                cridx = row_elem.idx + 1;
            }

            nnum_empty = num_empty + (self.rows() - cridx) * self.cols();
            if  nnum_empty > rand_idx {
                let pos = cridx * self.cols();
                let npos = (rand_idx - num_empty) + pos;
                let r = npos / self.cols();
                let c = npos % self.cols();
                return Some((c, r));
            }

            return None;
        }

        pub fn print_data(&self) {
            for row in &self.data {
                print!("{:<3}: ", row.idx);
                for elem in &row.data {
                    print!("([{:<3}] {:<3}) ", elem.idx, elem.data);
                }
                println!();
            }
        }
    }


    impl <T : std::fmt::Display + std::cmp::PartialEq> fmt::Display for SparseMatrix<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

            let mut ridx = 0;

            write!(f, "{:<4}|", "")?;
            for c in 0..self.cols() {
                write!(f, " {:<4} ", c)?;
            }
            writeln!(f, "")?;
            for _c in 0..self.cols()+1 {
                write!(f, "______")?;
            }
            writeln!(f, "")?;

            for row in &self.data {
                let cridx = row.idx;
                for r in ridx..cridx {
                    write!(f, "{:<4}|", r)?;
                    for _c in 0..self.cols() {
                        write!(f, " {:<4} ", self.def)?;
                    }
                    writeln!(f, "")?;
                }

                write!(f, "{:<4}|", cridx)?;
                let mut cidx = 0;
                for val in &row.data {
                    let ccidx = val.idx;
                    for _c in cidx..ccidx {
                        write!(f, " {:<4} ", self.def)?;
                    }
                    write!(f, " {:<4} ", val.data)?;
                    cidx = ccidx + 1;
                }

                for _c in cidx..self.rows() {
                    write!(f, " {:<4} ", self.def)?;
                }

                writeln!(f, "")?;

                ridx = cridx + 1;
            }

            for r in ridx..self.rows() {
                write!(f, "{:<4}|", r)?;
                for _c in 0..self.cols() {
                    write!(f, " {:<4} ", self.def)?;
                }
                writeln!(f, "")?;
            }

            Ok(())
        }
    }

    pub fn naive_print(mat : &SparseMatrix<u32>) {
    for r in 0..mat.rows() {
        for c in 0..mat.cols() {
            print!("{:<4} ", mat[[c, r]]);
        }
        println!();
    }
}


}



#[cfg(test)]
pub mod tests {
    use super::mat::*;
    // #[test]
    // fn mat_init() {
    //     let mat : SparseMatrix<u32> = SparseMatrix::new(10, 20, 0);
    //     assert_eq!(mat.rows(), 20);
    //     assert_eq!(mat.cols(), 10);
    // }

    // #[test]
    // fn mat_size_1() {
    //     let mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
    //     assert_eq!(mat.actual_size(), 0);
    // }

    // #[test]
    // fn mat_size_2() {
    //     let mut mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
    //     mat.insert(0, 0, 10);
    //     mat.insert(0, 1, 0);
    //     mat.insert(3, 1, 10);
    //     mat.insert(2, 0, 10);
    //     mat.insert(3, 2, 10);
    //     mat.insert(3, 1, 22);
    //     assert_eq!(mat.actual_size(), 4);
    // }

    // #[test]
    // fn mat_elements_1() {
    //     let mut mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
    //     mat.insert(0, 0, 10);
    //     mat.insert(0, 1, 0);
    //     mat.insert(3, 1, 10);
    //     mat.insert(2, 0, 12);
    //     mat.insert(3, 2, 10);
    //     mat.insert(3, 1, 22);
    //     assert_eq!(mat[[2,0]], 12);
    //     assert_eq!(mat[[3,1]], 22);
    //     assert_eq!(mat[[1,1]], 0);
    // }

    #[test]
    fn mat_avail() {
        let mut mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
        mat.insert(0, 0, 10);
        mat.insert(0, 1, 1);
        mat.insert(0, 2, 2);
        mat.insert(0, 3, 3);
        mat.insert(1, 0, 10);
        mat.insert(2, 0, 20);
        mat.insert(3, 0, 30);
        mat.insert(3, 1, 10);
        mat.insert(2, 0, 12);
        mat.insert(3, 2, 10);
        mat.insert(3, 1, 22);
        mat.insert(5, 8, 4);

        println!("{}", mat);
        naive_print(&mat);
        mat.print_data();

        for i in 0..10 {
            println!("Iter: {}", i);
            match mat.random_default() {
                Some((r, c)) => {
                    mat.insert(r, c, 99);
                    println!("\nInserting into: ({}, {})", r, c);
                }
                _ => { println!("None remaining"); break; }
            };
        }

        println!("{}", mat);

        assert_eq!(mat[[2,0]], 12);
        assert_eq!(mat[[3,1]], 22);
        assert_eq!(mat[[3,2]], 10);
    }

}
