/// String similarity and suggestion utilities for better error messages
use std::cmp::min;

/// Calculate the Levenshtein distance between two strings.
/// This measures the minimum number of single-character edits (insertions, deletions, or substitutions)
/// needed to transform one string into another.
///
/// # Examples
///
/// ```
/// use livac::suggestions::levenshtein_distance;
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
/// assert_eq!(levenshtein_distance("hello", "hello"), 0);
/// ```
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Create a matrix to store distances
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // Fill in the rest of the matrix
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

            matrix[i][j] = min(
                min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1,      // insertion
                ),
                matrix[i - 1][j - 1] + cost,   // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Find the best suggestion from a list of candidates based on string similarity.
/// Returns the most similar string if it's within the threshold, otherwise None.
///
/// # Arguments
///
/// * `input` - The misspelled or incorrect input
/// * `candidates` - List of valid options to compare against
/// * `max_distance` - Maximum allowed edit distance (default: 3 for good suggestions)
///
/// # Examples
///
/// ```
/// use livac::suggestions::find_suggestion;
/// let vars: Vec<String> = vec!["userName".into(), "userAge".into(), "userId".into()];
/// assert_eq!(find_suggestion("usrName", &vars, 2), Some("userName".to_string()));
/// assert_eq!(find_suggestion("xyz", &vars, 2), None);
/// ```
pub fn find_suggestion(input: &str, candidates: &[String], max_distance: usize) -> Option<String> {
    let mut best_match: Option<(String, usize)> = None;

    for candidate in candidates {
        let distance = levenshtein_distance(input, candidate);

        if distance <= max_distance {
            match &best_match {
                None => best_match = Some((candidate.clone(), distance)),
                Some((_, best_distance)) => {
                    if distance < *best_distance {
                        best_match = Some((candidate.clone(), distance));
                    }
                }
            }
        }
    }

    best_match.map(|(name, _)| name)
}

/// Find multiple suggestions (up to N) sorted by similarity.
/// Useful when you want to show several options to the user.
///
/// # Arguments
///
/// * `input` - The misspelled or incorrect input
/// * `candidates` - List of valid options to compare against
/// * `max_distance` - Maximum allowed edit distance
/// * `max_suggestions` - Maximum number of suggestions to return (default: 3)
///
/// # Examples
///
/// ```
/// use livac::suggestions::find_multiple_suggestions;
/// let funcs: Vec<String> = vec!["calculate".into(), "calibrate".into(), "consolidate".into()];
/// let suggestions = find_multiple_suggestions("calulate", &funcs, 3, 2);
/// // Returns ["calculate", "calibrate"]
/// ```
pub fn find_multiple_suggestions(
    input: &str,
    candidates: &[String],
    max_distance: usize,
    max_suggestions: usize,
) -> Vec<String> {
    let mut matches: Vec<(String, usize)> = candidates
        .iter()
        .map(|candidate| {
            let distance = levenshtein_distance(input, candidate);
            (candidate.clone(), distance)
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance (closest first)
    matches.sort_by_key(|(_, distance)| *distance);

    // Take top N suggestions
    matches
        .into_iter()
        .take(max_suggestions)
        .map(|(name, _)| name)
        .collect()
}

/// Calculate similarity score as a percentage (0-100).
/// Higher score means more similar strings.
///
/// # Examples
///
/// ```
/// use livac::suggestions::similarity_score;
/// assert_eq!(similarity_score("hello", "hello"), 100);
/// assert!(similarity_score("kitten", "sitting") > 50);
/// assert!(similarity_score("abc", "xyz") < 30);
/// ```
pub fn similarity_score(a: &str, b: &str) -> u8 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 100;
    }

    let distance = levenshtein_distance(a, b);
    let similarity = 100.0 * (1.0 - (distance as f64 / max_len as f64));

    similarity.max(0.0).min(100.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "hxllo"), 1);
    }

    #[test]
    fn test_find_suggestion() {
        let candidates = vec![
            "userName".to_string(),
            "userAge".to_string(),
            "userId".to_string(),
            "productName".to_string(),
        ];

        // Close match
        assert_eq!(
            find_suggestion("usrName", &candidates, 2),
            Some("userName".to_string())
        );

        // Exact match
        assert_eq!(
            find_suggestion("userId", &candidates, 2),
            Some("userId".to_string())
        );

        // Too far
        assert_eq!(find_suggestion("xyz", &candidates, 2), None);

        // Multiple close matches - should pick closest
        let result = find_suggestion("userNam", &candidates, 3);
        assert!(result == Some("userName".to_string()));
    }

    #[test]
    fn test_find_multiple_suggestions() {
        let candidates = vec![
            "calculate".to_string(),
            "calibrate".to_string(),
            "consolidate".to_string(),
            "coordinate".to_string(),
        ];

        let suggestions = find_multiple_suggestions("calulate", &candidates, 3, 2);
        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.contains(&"calculate".to_string()));
    }

    #[test]
    fn test_similarity_score() {
        assert_eq!(similarity_score("hello", "hello"), 100);
        assert!(similarity_score("hello", "hallo") > 75); // 1 char difference in 5 chars = 80%
        assert!(similarity_score("hello", "xyz") < 40);
        assert_eq!(similarity_score("", ""), 100);
    }

    #[test]
    fn test_common_typos() {
        let candidates = vec![
            "length".to_string(),
            "push".to_string(),
            "pop".to_string(),
            "print".to_string(),
        ];

        // Common typo: missing letter
        assert_eq!(
            find_suggestion("lenght", &candidates, 2),
            Some("length".to_string())
        );

        // Common typo: swapped letters
        assert_eq!(
            find_suggestion("pint", &candidates, 2),
            Some("print".to_string())
        );

        // Common typo: extra letter
        assert_eq!(
            find_suggestion("pussh", &candidates, 2),
            Some("push".to_string())
        );
    }
}
