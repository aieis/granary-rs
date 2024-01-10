pub mod mat {
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
        let mut cb = 0;
        let mut ce = svec.len();

        if ce == 0 {
            return (false, 0);
        }
        
        loop {
            let cm = (ce - cb) / 2;

            if svec[cm].idx == cidx {
                return (true, cm)
            }
            if cm <= cb {
                return (false, cm);
            }

            if svec[cm].idx > cidx {
                ce = cm;
            } else {
                cb = cm;
            }
        }
    }


    impl <T : std::cmp::PartialEq> SparseMatrix<T> {
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
                    idx : cidx,
                    data : val
                });
            }
        }
    }

    // pub fn smat_find_all<T : std::cmp::PartialEq>(mat: &SparseMatrix<T>, val : T, cidx : usize, ridx : usize, radius : usize) -> Vec<T> {
    //     let cf = if cidx >= radius {cidx - radius} else {0};
    //     let ct = if cidx + radius < mat.cols() {cidx + radius} else {mat.cols() - 1};

    //     let rf = if ridx >= radius {ridx - radius} else {0};
    //     let rt = if ridx + radius < mat.rows() {ridx + radius} else {mat.rows() - 1};

    //     let mut res : Vec<T> = vec![];
    //     panic!("Not Implemented!");
    //     return res;
    // }
}


#[cfg(test)]
pub mod tests {
    use super::mat::*;
    #[test]
    fn mat_init() {
        let mat : SparseMatrix<u32> = SparseMatrix::new(10, 20, 0);
        assert_eq!(mat.rows(), 20);
        assert_eq!(mat.cols(), 10);
    }

    #[test]
    fn mat_size_1() {
        let mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
        assert_eq!(mat.actual_size(), 0);
    }

    #[test]
    fn mat_size_2() {
        let mut mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
        mat.insert(0, 0, 10);
        mat.insert(0, 1, 0);
        mat.insert(3, 1, 10);
        mat.insert(2, 0, 10);
        mat.insert(3, 2, 10);
        mat.insert(3, 1, 22);
        assert_eq!(mat.actual_size(), 4);
    }

    #[test]
    fn mat_elements_1() {
        let mut mat : SparseMatrix<u32> = SparseMatrix::new(10, 10, 0);
        mat.insert(0, 0, 10);
        mat.insert(0, 1, 0);
        mat.insert(3, 1, 10);
        mat.insert(2, 0, 12);
        mat.insert(3, 2, 10);
        mat.insert(3, 1, 22);
        assert_eq!(mat[[2,0]], 12);
        assert_eq!(mat[[3,1]], 22);
        assert_eq!(mat[[1,1]], 0);
    }    
}
