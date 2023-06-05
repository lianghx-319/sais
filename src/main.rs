use sais::construct_suffix_array_parallel;

fn main() {
    let text = "banana";
    let sa = construct_suffix_array_parallel(text.as_bytes());
    println!("Suffix array for '{}': {:?}", text, sa);
    assert_eq!(sa, vec![5, 3, 1, 0, 4, 2]);
}
