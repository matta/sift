pub(crate) fn midpoint(left: &str, right: &str) -> String {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut mid = String::new();

    if !left.iter().all(|&b| b.is_ascii_lowercase())
        || !right.iter().all(|&b| b.is_ascii_lowercase())
        || (!left.is_empty() && !right.is_empty() && left >= right)
    {
        return mid;
    }

    let mut left = left.iter().copied().chain(std::iter::repeat(b'a' - 1));
    let mut right = right.iter().copied().chain(std::iter::repeat(b'z' + 1));

    let mut l = 0;
    let mut r = 0;

    while l == r {
        l = left.next().unwrap();
        r = right.next().unwrap();
        if l == r {
            mid.push(l as char);
        }
    }

    if l == b'a' - 1 {
        while r == b'a' {
            mid.push('a');
            r = right.next().unwrap();
        }
        if r == b'b' {
            mid.push('a');
            r = b'z' + 1;
        }
    } else if l + 1 == r {
        r = b'z' + 1;
        mid.push(l as char);
        loop {
            l = left.next().unwrap();
            if l != b'z' {
                break;
            }
            mid.push('z');
        }
    }
    mid.push((r - (r - l) / 2) as char);

    mid
}

#[cfg(test)]
mod tests {
    use super::midpoint;

    #[test]
    fn test_base_case() {
        let mid = midpoint("", "");
        assert_eq!(mid, "n");
    }

    #[test]
    fn test_leftmost() {
        let mid = midpoint("", "n");
        assert_eq!(mid, "g");
    }

    #[test]
    fn test_rightmost() {
        let mid = midpoint("n", "");
        assert_eq!(mid, "u");
    }

    #[test]
    fn test_invalid_left_not_ascii() {
        let mid = midpoint("𞹔", "");
        assert_eq!(mid, "");
    }
    #[test]
    fn test_invalid_right_not_ascii() {
        let mid = midpoint("", "𞹔");
        assert_eq!(mid, "");
    }

    #[test]
    fn test_invalid_equal() {
        let mid = midpoint("a", "a");
        assert_eq!(mid, "");
    }

    #[test]
    fn test_basic_one() {
        let left = "abcde";
        let right = "abchi";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcf");
    }

    #[test]
    fn test_basic_two() {
        let left = "abc";
        let right = "abchi";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcd");
    }

    #[test]
    fn test_consecutive_one() {
        let left = "abhs";
        let right = "abit";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abhw");
    }

    #[test]
    fn test_consecutive_two() {
        let left = "abh";
        let right = "abit";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abhn");
    }

    #[test]
    fn test_consecutive_z_one() {
        let left = "abhz";
        let right = "abit";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abhzn");
    }

    #[test]
    fn test_consecutive_z_two() {
        let left = "abhzs";
        let right = "abit";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abhzw");
    }

    #[test]
    fn test_consecutive_z_three() {
        let left = "abhzz";
        let right = "abit";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abhzzn");
    }

    #[test]
    fn test_right_is_a_or_b_one() {
        let left = "abc";
        let right = "abcah";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcad");
    }

    #[test]
    fn test_right_is_a_or_b_two() {
        let left = "abc";
        let right = "abcab";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcaan");
    }

    #[test]
    fn test_right_is_a_or_b_three() {
        let left = "abc";
        let right = "abcaah";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcaad");
    }

    #[test]
    fn test_right_is_a_or_b_four() {
        let left = "abc";
        let right = "abcb";
        let mid = midpoint(left, right);
        assert_eq!(mid, "abcan");
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn does_not_crash(left in "\\PC*", right in "\\PC*") {
            midpoint(&left, &right);
        }

        #[test]
        fn works(a in "[a-z]*", b in "[a-z]+") {
            works_impl(a, b);
        }
    }

    fn works_impl(a: String, b: String) {
        if a == b {
            return;
        }
        let mid = midpoint(&a, &b);
        assert_ne!(a, mid);
        assert_ne!(b, mid);
        if a >= b {
            assert_eq!(mid, "");
        } else {
            assert!(a < mid);
            if !b.is_empty() {
                assert!(mid < b);
            }
            // The midpoint should never be a string ending in 'a'.
            assert_ne!(mid.chars().last().unwrap(), 'a');
        }
    }
}
