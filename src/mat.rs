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
            let (ce, ci) = svec_find_idx(&self.data, idx[0]);
            if !ce {
                return &self.def;
            }

            let (re, ri) = svec_find_idx(&self.data[ci].data, idx[1]);
            if !re {
                return &self.def;
            }

            return &self.data[ci].data[ri].data;
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

            let (ce, ci) = svec_find_idx(&self.data, cidx);
            if !ce && val != self.def {
                self.data.insert(ci, Element {
                    idx : cidx,
                    data :vec![Element { idx: ridx, data : val}]
                });

                return;
            }

            let (re, ri) = svec_find_idx(&self.data[ci].data, ridx);

            if re {
                if val == self.def {
                    self.data[ci].data.remove(ri);
                    if self.data[ci].data.len() == 0 {
                        self.data.remove(ci);
                    }
                    return;
                }
                
                self.data[ci].data[ri].data = val;
                return;
            }

            if !re && val != self.def {
                self.data[ci].data.insert(ri, Element {
                    idx : ri,
                    data : val
                });
            }
        }

        pub fn random_default(&self) -> Option<(usize, usize)>{
            let total = self.rows() * self.cols() - self.actual_size();

            if total == 0 {
                return None;
            }

            let rand_idx = 0 ;//rand::Rng::gen_range(&mut rand::thread_rng(), 0..total);
            println!("Default index {} from {}", rand_idx, total);

            let mut cridx = 0;
            let mut ccidx = 0;

            let mut num_empty = 0;
            let mut nnum_empty;
            
            for row_elem in &self.data {
                println!("{} {}", num_empty, ccidx);
                print!("Row ({:2}) {:2}: ", cridx, row_elem.idx);
                nnum_empty = num_empty + (row_elem.idx - cridx) * self.cols();
                if  nnum_empty > rand_idx {
                    let pos = cridx * self.cols();
                    let npos = (rand_idx - num_empty) + pos;
                    let r = npos / self.cols();
                    let c = npos % self.cols();
                    return Some((r, c));
                }

                num_empty = nnum_empty;
                cridx = row_elem.idx;
                ccidx = 0;
                
                for elem in &row_elem.data {
                    print!(" ({}, {}) ", elem.idx, elem.data);
                }
                println!("");

                for elem in &row_elem.data {
                    nnum_empty = num_empty + (elem.idx - ccidx);
                    if  nnum_empty > rand_idx {
                        let c = rand_idx - num_empty + ccidx;
                        return Some((cridx, c));
                    }
                    num_empty = nnum_empty;
                    ccidx = elem.idx + 1;
                }

                nnum_empty = num_empty + (self.cols() - ccidx);

                if nnum_empty > rand_idx {
                    let c = rand_idx - num_empty + ccidx;
                    return Some((cridx, c));
                }

                num_empty = nnum_empty;
                cridx = row_elem.idx + 1;
            }

            nnum_empty = num_empty + (self.rows - cridx) * self.cols() - ccidx;
            if  nnum_empty > rand_idx {
                let pos = cridx * self.cols() + ccidx;
                let npos = (rand_idx - num_empty + 1) + pos;
                let r = npos / self.cols();
                let c = npos % self.cols();
                return Some((r, c));
            }
            
            
            return None;
        }
    }

    
    impl <T : std::fmt::Display + std::cmp::PartialEq> fmt::Display for SparseMatrix<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

            let mut ridx = 0;
            
            for row in &self.data {
                let cridx = row.idx;
                for _r in ridx..cridx {
                    for _c in 0..self.cols() {
                        write!(f, " {} ", self.def)?;                        
                    }
                    writeln!(f, "")?;
                }

                let mut cidx = 0;
                for val in &row.data {
                    let ccidx = val.idx;
                    for _c in cidx..ccidx {
                        write!(f, " {} ", self.def)?;
                    }
                    write!(f, " {} ", val.data)?;
                    cidx = ccidx + 1;
                }

                for _c in cidx..self.rows() {
                    write!(f, " {} ", self.def)?;
                }

                writeln!(f, "")?;
                
                ridx = cridx + 1;
            }

            for _r in ridx..self.rows() {
                for _c in 0..self.cols() {
                    write!(f, " {} ", self.def)?;
                }
                writeln!(f, "")?;
            }

            Ok(())
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
        mat.insert(0, 1, 0);
        mat.insert(3, 1, 10);
        mat.insert(2, 0, 12);
        mat.insert(3, 2, 10);
        mat.insert(3, 1, 22);

        for i in 0..29 {
            println!("Iter: {}", i);
            match mat.random_default() {
                Some((r, c)) => {
                    mat.insert(r, c, 99);
                    println!("({}, {})", r, c);
                }
                _ => { println!("None remaining"); break; }
            };
        }

        println!("{}", mat);
        
        assert_eq!(mat[[2,0]], 12);
        assert_eq!(mat[[3,1]], 22);
    }

}
