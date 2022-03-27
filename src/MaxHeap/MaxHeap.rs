use std::collections::VecDeque;

pub struct MaxHeap<K, V> {
    data: VecDeque<(K, V)>,
}

impl<K: PartialOrd, V> MaxHeap<K, V> {
    fn _parent_idx(&self, idx: usize) -> Option<usize> {
        if idx == 0 {
            None
        } else {
            Some((idx - 1) / 2)
        }
    }

    fn _left_idx(&self, idx: usize) -> usize {
        idx * 2 + 1
    }

    fn _right_idx(&self, idx: usize) -> usize {
        idx * 2 + 2
    }

    fn _sift_up(&mut self, mut idx: usize) {
        loop {
            if idx == 0 {
                break;
            }
            let op_parent_idx = self._parent_idx(idx);
            match op_parent_idx {
                None => {
                    break;
                }
                Some(parent_idx) => {
                    if self.data[parent_idx].0 < self.data[idx].0 {
                        self.data.swap(parent_idx, idx);
                        idx = parent_idx;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn _sift_down(&mut self, mut idx: usize) {
        loop {
            let left_idx = self._left_idx(idx);
            let right_idx = self._right_idx(idx);
            let has_left = left_idx < self.data.len();
            let has_right = right_idx < self.data.len();

            if !has_left && !has_right {
                break;
            } else if !has_left && has_right {
                break;
            } else if has_left && !has_right {
                if self.data[idx].0 < self.data[left_idx].0 {
                    self.data.swap(idx, left_idx);
                    break;
                } else {
                    break;
                }
            } else {
                let max_child_idx = if self.data[left_idx].0 > self.data[right_idx].0 {
                    left_idx
                } else {
                    right_idx
                };

                if self.data[idx].0 < self.data[max_child_idx].0 {
                    self.data.swap(idx, max_child_idx);
                    idx = max_child_idx;
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}

impl<K: PartialOrd, V> MaxHeap<K, V> {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn new() -> Self {
        MaxHeap {
            data: VecDeque::<(K, V)>::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.data.push_back((k, v));
        self._sift_up(self.len() - 1);
    }

    pub fn pop_max(&mut self) -> Option<(K, V)> {
        if self.len() == 0 {
            None
        } else {
            // first swap then pop out
            self.data.swap(0, self.len() - 1);
            self.data.pop_back().map(|o| {
                self._sift_down(0);
                o
            })
        }
    }
}
