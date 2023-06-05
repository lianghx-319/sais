use rayon::prelude::*;

extern crate num_cpus;

// A struct representing a suffix, consisting of a start index and a length
struct Suffix<'a> {
    start_index: usize,
    length: usize,
    text: &'a [u8],
}

impl<'a> Clone for Suffix<'a> {
    fn clone(&self) -> Self {
        Self {
            start_index: self.start_index,
            length: self.length,
            text: self.text.clone(),
        }
    }
}

// A function for comparing two suffixes based on their corresponding substrings
fn suffix_compare(s1: &Suffix, s2: &Suffix) -> std::cmp::Ordering {
    s1.text[s1.start_index..]
        .cmp(&s2.text[s2.start_index..])
        .then_with(|| s1.length.cmp(&s2.length))
}

// A function for constructing the suffix array in parallel using parallel multi-way mergesort
pub fn construct_suffix_array_parallel(text: &[u8]) -> Vec<usize> {
    // Construct an array of suffixes from the input text
    let suffixes: Vec<Suffix> = (0..text.len())
        .map(|i| Suffix {
            start_index: i,
            length: text.len() - i,
            text,
        })
        .collect();

    // Sort the suffixes in parallel using parallel multi-way mergesort
    let num_threads = num_cpus::get();
    let chunk_size = if suffixes.len() < num_threads {
        suffixes.len()
    } else {
        suffixes.len() / num_threads
    };
    let sorted_suffixes: Vec<Vec<Suffix>> = suffixes
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut sorted_chunk = chunk.to_vec();
            // TODO: replace a more effective sort method
            sorted_chunk.sort_by(suffix_compare);
            sorted_chunk
        })
        .collect();

    // Merge the sorted suffixes together using parallel multi-way mergesort
    let mut suffix_array: Vec<usize> = Vec::with_capacity(text.len());
    let mut indices: Vec<usize> = vec![0; sorted_suffixes.len()];
    while suffix_array.len() < text.len() {
        let mut min_suffix: Option<&Suffix> = None;
        let mut min_index: Option<usize> = None;
        for (i, chunk) in sorted_suffixes.iter().enumerate() {
            if indices[i] < chunk.len() {
                let suffix = &chunk[indices[i]];
                if min_suffix.is_none()
                    || suffix_compare(suffix, min_suffix.unwrap()) == std::cmp::Ordering::Less
                {
                    min_suffix = Some(suffix);
                    min_index = Some(i);
                }
            }
        }
        if let Some(suffix) = min_suffix {
            suffix_array.push(suffix.start_index);
            indices[min_index.unwrap()] += 1;
        }
    }

    suffix_array
}

pub fn lcp_array_parallel(sa: &[usize], a: &[u8]) -> Vec<usize> {
    let n = sa.len();
    let mut rank = vec![0; n];
    // Compute rank array
    for i in 0..n {
        rank[sa[i]] = i;
    }
    // Compute LCP array in parallel
    (1..n)
        .into_par_iter()
        .map(|j| {
            let mut h = 0;
            let k = sa[rank[j] - 1];
            while j + h < n && k + h < n && a[j + h] == a[k + h] {
                h += 1;
            }
            h
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_sa() {
        let text = "banana";
        let sa = construct_suffix_array_parallel(text.as_bytes());
        println!("Suffix array for '{}': {:?}", text, sa);
        assert_eq!(sa, vec![5, 3, 1, 0, 4, 2]);
    }
}
