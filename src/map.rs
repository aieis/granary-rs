use std::ops;

mod map {
    struct Element<T> {
        idx : usize,
        data : T
    }

    type SparseVector<T> = Vec<Element<T>>;

    struct SparseMatrix<T> {
        cols : usize,
        rows : usize,
        data : SparseVector<SparseVector<T>>
    }
    

    impl<T> std::ops::Index<[usize;2]> for SparseMatrix<T> {
        type Output = T;
        fn index(&self, idx : [usize;2]) -> &T {
            return &self.data[idx[0]].data[idx[0]].data;
        }
    }


    Fn insert_matrix<T>(mat : &mut SparseMatrix<T>, cidx : usize, ridx : usize, val : T )
    {
        let mut cb = 0;
        let mut ce = mat.len();
        loop {
            let cm = (ce - cb) / 2;

            if mat[cm].idx == cidx {
                let row = &mat[cm].data;
                let mut rb = 0;
                let mut re = row.len();
                loop {
                    let rm = (re - rb) / 2;

                    if row[rm].idx == ridx {
                        mat[cm].data[rm].data = val;
                        return;
                    }

                    if cm <= cb {
                        mat[cm].data.insert(cm, Element {
                            idx : cidx,
                            data : val
                        });

                        return;
                    }

                    if mat[cm].idx > cidx {
                        re = rm;
                    } else {
                        rb = rm;
                    }
                };

            }

            if cm <= cb {
                mat.insert(cm, Element {
                    idx : cidx,
                    data :vec![Element { idx: ridx, data : val}]
                });

                break;
            }

            if mat[cm].idx > cidx {
                ce = cm;
            } else {
                cb = cm;
            }
        };
    }
}
