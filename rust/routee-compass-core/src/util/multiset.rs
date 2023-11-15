pub struct MultiSet<'a, T: Clone + Copy> {
    sets: &'a Vec<Vec<T>>,
    pos: Option<Vec<usize>>,
    final_pos: Vec<usize>,
}

impl<'a, T> From<&'a Vec<Vec<T>>> for MultiSet<'a, T>
where
    T: Clone + Copy,
{
    fn from(sets: &'a Vec<Vec<T>>) -> Self {
        let final_pos: Vec<usize> = sets.iter().map(|v| v.len() - 1).collect();
        let pos: Option<Vec<usize>> = Some(vec![0; sets.len()]);
        MultiSet {
            sets,
            pos,
            final_pos,
        }
    }
}

impl<'a, T> Iterator for MultiSet<'a, T>
where
    T: Clone + Copy,
{
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.pos {
            None => None,
            Some(position) => {
                let result: Vec<T> = position
                    .iter()
                    .zip(0..self.sets.len())
                    .map(|(j, i)| self.sets[i][*j])
                    .collect();
                let mut next_pos = position.to_vec();
                // tick through full table of index combinations
                let mut finished = false;
                for idx in 0..self.sets.len() {
                    if next_pos[idx] < self.final_pos[idx] {
                        next_pos[idx] += 1;
                        break;
                    } else if idx == self.sets.len() - 1 {
                        finished = true;
                        break;
                    } else {
                        for r in next_pos.iter_mut().take(idx + 1) {
                            *r = 0;
                        }
                    }
                }
                if finished {
                    self.pos = None;
                } else {
                    self.pos = Some(next_pos);
                }
                Some(result)
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::MultiSet;

    #[test]
    fn test_multiset() {
        let input = vec![vec![1, 3], vec![2], vec![5, 7, 9]];
        let ms = MultiSet::from(&input);
        let result: Vec<Vec<i32>> = ms.into_iter().collect();
        let expected = [
            [1, 2, 5],
            [3, 2, 5],
            [1, 2, 7],
            [3, 2, 7],
            [1, 2, 9],
            [3, 2, 9],
        ];
        assert_eq!(result, expected);
    }
}
