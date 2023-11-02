use crate::errors::errors::ValidationError;

/// Given a `(depth, offset)`, calculates and returns the corresponding index.
///
/// # Arguments
///
/// * `depth` - The depth of the node.
/// * `offset` - The offset of the node within its depth.
///
/// # Returns
///
/// * usize containing the calculated index.
pub fn depth_offset_to_index(depth: usize, offset: usize) -> Result<usize, ValidationError> {
    let base = (1 << depth) - 1;
    if offset > base {
        return Err(ValidationError::Invalid);
    }
    Ok(base + offset)
}

/// Given an index, returns its `(depth, offset)`.
///
/// # Arguments
///
/// * `index` - The index of the node.
///
/// # Returns
///
/// * A tuple `(depth, offset)`.
pub fn index_to_depth_offset(index: usize) -> (usize, usize) {
    let mut depth = 0;
    let mut nodes_at_current_depth = 1;

    while index >= nodes_at_current_depth {
        depth += 1;
        nodes_at_current_depth += 1 << depth;
    }

    let base = if depth == 0 { 0 } else { (1 << depth) - 1 };
    let offset = index - base;

    (depth, offset)
}

/// Given an index, returns the index of its parent.
///
/// # Arguments
///
/// * `index` - The index of the node.
///
/// # Returns
///
/// * An integer representing the index of the parent node.
pub fn parent_index(index: usize) -> Option<usize> {
    // TODO: Implement the function.
    if index == 0 {
        return None;
    }
    return Some((index - 1) / 2);
}

/// Given an index, returns the index of its left-most child.
///
/// # Arguments
///
/// * `index` - The index of the node.
///
/// # Returns
///
/// * An integer representing the index of the left-most child node.
pub fn left_child_index(index: usize) -> usize {
    return (index * 2) + 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_offset_to_index() {
        assert_eq!(depth_offset_to_index(0, 0), Ok(0));
        assert_eq!(depth_offset_to_index(1, 0), Ok(1));
        assert_eq!(depth_offset_to_index(1, 1), Ok(2));
        assert_eq!(depth_offset_to_index(2, 3), Ok(6));
        assert_eq!(depth_offset_to_index(5, 0), Ok(31));
        assert_eq!(depth_offset_to_index(7, 0), Ok(127));
        assert_eq!(depth_offset_to_index(10, 0), Ok(1023));
        assert_eq!(depth_offset_to_index(10, 10), Ok(1033));
        // edge cases
        assert_eq!(depth_offset_to_index(2, 5), Err(ValidationError::Invalid));
        assert_eq!(depth_offset_to_index(1, 2), Err(ValidationError::Invalid));
    }

    #[test]
    fn test_index_to_depth_offset() {
        assert_eq!(index_to_depth_offset(0), (0, 0));
        assert_eq!(index_to_depth_offset(1), (1, 0));
        assert_eq!(index_to_depth_offset(2), (1, 1));
        assert_eq!(index_to_depth_offset(7), (3, 0));
        assert_eq!(index_to_depth_offset(7), (3, 0));
        assert_eq!(index_to_depth_offset(14), (3, 7)); // Last leaf node.
        assert_eq!(index_to_depth_offset(15), (4, 0)); // Just after the last leaf node.
        assert_eq!(index_to_depth_offset(16), (4, 1)); // The index after the previous.
    }

    #[test]
    fn test_parent_index() {
        assert_eq!(parent_index(0), None);
        assert_eq!(parent_index(1), Some(0));
        assert_eq!(parent_index(2), Some(0));
        assert_eq!(parent_index(5), Some(2));
        assert_eq!(parent_index(3), Some(1));
        assert_eq!(parent_index(7), Some(3));
        assert_eq!(parent_index(14), Some(6));
    }

    #[test]
    fn test_left_child_index() {
        assert_eq!(left_child_index(0), 1);
        assert_eq!(left_child_index(1), 3);
        assert_eq!(left_child_index(5), 11);
    }
}
