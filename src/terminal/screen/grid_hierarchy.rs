use std::mem;

use self::GridHierarchy::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SplitKind {
    Horizontal(u32),
    Vertical(u32),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GridHierarchy {
    Grid(u64),
    Split {
        tag: u64,
        kind: SplitKind,
        left: Box<GridHierarchy>,
        right: Box<GridHierarchy>,
    },
}

impl GridHierarchy {
    
    pub fn find_first_grid(&self, tag: u64) -> Option<u64> {
        fn is_grid(grid: &GridHierarchy) -> u64 {
            match *grid {
                Grid(tag) => tag,
                Split { ref left, .. } => is_grid(left),
            }
        }
        self.find(tag).map(is_grid)
    }

    pub fn remove(&mut self, remove: u64) -> Option<(u64, SplitKind)> {
        if let Some((new_grid, k)) = if let Split { ref left, ref right, kind, .. } = *self {
            match (left, right) {
                (&box Grid(tag), _) | (&box Split { tag, .. }, _) 
                    if tag == remove => Some((*right.clone(), kind)),
                (_, &box Grid(tag)) | (_, &box Split { tag, .. })
                    if tag == remove => Some((*left.clone(), kind)),
                _                                   => None
            }
        } else { None } {
            let tag = new_grid.tag();
            mem::replace(self, new_grid);
            Some((tag, k))
        } else if let Split { ref mut left, ref mut right, .. } = *self {
            left.remove(remove).or_else(move || right.remove(remove))
        } else { None }
    }

    pub fn replace(&mut self, tag: u64, new: GridHierarchy) {
        self.find_mut(tag).map(|grid| *grid = new);
    }

    fn tag(&self) -> u64 {
        match *self {
            Grid(tag) | Split { tag, .. } => tag
        }
    }

    fn find(&self, id: u64) -> Option<&GridHierarchy> {
        match *self {
            Grid(tag) | Split { tag, .. } if id == tag => Some(self),
            Split { ref left, ref right, .. } => {
                left.find(id).or_else(move || right.find(id))
            }
            _ => None
        }
    }

    fn find_mut(&mut self, id: u64) -> Option<&mut GridHierarchy> {
        match *self {
            Grid(tag) | Split { tag, .. } if id == tag => Some(self),
            Split { ref mut left, ref mut right, .. } => {
                left.find_mut(id).or_else(move || right.find_mut(id))
            }
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::GridHierarchy::*;

    // The hierarchy this sets up is:
    //  0
    //  | \
    //  1  2
    //  | \
    //  3 0x0beefdad
    // Beef Dad is the needle for these tests.
    fn setup_grid_hierarchy() -> GridHierarchy {
        Split {
            tag: 0,
            kind: SplitKind::Horizontal(2),
            left: Box::new(Split {
                tag: 1,
                kind: SplitKind::Horizontal(2),
                left: Box::new(Grid(3)),
                right: Box::new(Grid(0x0beefdad)),
            }),
            right: Box::new(Grid(2)),
        }
    }

    // After this test:
    // 0
    // | \
    // 3  2
    #[test]
    fn remove_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.remove(0x0beefdad);
        assert_eq!(gh, Split {
            tag: 0,
            kind: SplitKind::Horizontal(2),
            left: Box::new(Grid(3)),
            right: Box::new(Grid(2)),
        })
    }

    // After this test:
    // 0
    // | \
    // 1  2
    // | \
    // 3 0x0badcafe
    #[test]
    fn replace_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.replace(0x0beefdad, GridHierarchy::Grid(0x0badcafe));
        assert_eq!(gh, Split {
            tag: 0,
            kind: SplitKind::Horizontal(2),
            left: Box::new(Split {
                tag: 1,
                kind: SplitKind::Horizontal(2),
                left: Box::new(Grid(3)),
                right: Box::new(Grid(0x0badcafe)),
            }),
            right: Box::new(Grid(2)),
        })
    }
}
