use rayon::prelude::*;
use std::cmp::Ordering;
use std::sync::Mutex;

extern crate num_cpus;

// A struct representing a suffix, consisting of a start index and a length
#[derive(PartialEq, Eq)]
struct Suffix<'a> {
    start_index: usize,
    text: &'a [u8],
}

impl<'a> PartialOrd for Suffix<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.text[self.start_index..].partial_cmp(&other.text[other.start_index..])
    }
}

impl<'a> Ord for Suffix<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.text[self.start_index..].cmp(&other.text[other.start_index..])
    }
}

// A function for constructing the suffix array in parallel
pub fn construct_suffix_array_parallel(text: &[u8]) -> Vec<usize> {
    let mut suffixes: Vec<_> = (0..text.len())
        .map(|i| Suffix {
            start_index: i,
            text,
        })
        .collect();

    suffixes.par_sort_unstable_by(|a, b| a.cmp(b));
    // Sort the suffixes in parallel
    let sorted_suffixes: Vec<_> = suffixes
        .into_iter()
        .map(|suffix| suffix.start_index)
        .collect();

    sorted_suffixes
}

pub fn lcp_array_parallel(sa: &[usize], a: &[u8]) -> Vec<usize> {
    let n = sa.len();
    let mut lcp = vec![0; n - 1]; // One less than the length of the input
    let mut rank = vec![0; n];

    for i in 0..n {
        rank[sa[i]] = i;
    }

    let lcp_ref = Mutex::new(&mut lcp);
    let rank_ref = &rank;

    sa.par_iter().enumerate().skip(1).for_each(|(_i, &suf)| {
        let rank = rank_ref[suf];
        let prev_suf = sa[rank - 1];
        let mut length = 0;
        while suf + length < n && prev_suf + length < n && a[suf + length] == a[prev_suf + length] {
            length += 1;
        }
        lcp_ref.lock().unwrap()[rank - 1] = length;
    });

    lcp
}

// Serial implementation of LCP array construction for validation
pub fn lcp_array_serial(sa: &[usize], s: &[u8]) -> Vec<usize> {
    let n = sa.len();
    let mut lcp = vec![0; n - 1]; // One less than the number of suffixes
    let mut rank = vec![0; n];

    for i in 0..n {
        rank[sa[i]] = i;
    }

    let mut h = 0;
    for i in 0..n {
        if rank[i] > 0 {
            let k = sa[rank[i] - 1];
            while i + h < n && k + h < n && s[i + h] == s[k + h] {
                h += 1;
            }
            lcp[rank[i] - 1] = h;
            if h > 0 {
                h -= 1;
            }
        }
    }
    lcp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_suffix_array_parallel() {
        let text = "banana".as_bytes();
        let sa = construct_suffix_array_parallel(text);
        assert_eq!(sa, vec![5, 3, 1, 0, 4, 2]);
    }

    #[test]
    fn test_lcp_array_parallel() {
        let text = "banana".as_bytes();
        let sa = construct_suffix_array_parallel(text);
        let lcp_parallel = lcp_array_parallel(&sa, text);
        let lcp_serial = lcp_array_serial(&sa, text);
        // The expected LCP array for "banana" with sa [5, 3, 1, 0, 4, 2] is [1, 3, 0, 0, 2]
        assert_eq!(lcp_parallel, lcp_serial); // The last element is always 0 as there is no suffix after the last one.
    }
}
