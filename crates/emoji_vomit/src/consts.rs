//! Some consts

use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;

lazy_static! {
    /// Words to ignore when matching to an emoji.
    /// These are mostly short, commonly used words like "is", "by", etc.
    pub static ref IGNORE_WORDS: HashSet<&'static str> = HashSet::from([
        "the", "of", "and", "to", "in", "a", "is", "that", "it", "he", "was", "for",
        "on", "are", "as", "with", "his", "they", "I", "at", "be", "this", "have",
        "from", "or", "one", "had", "by", "word", "but", "not", "what", "all",
        "were", "we", "when", "your", "can", "said", "te", "use", "an", "each",
        "which", "she", "do", "how", "their", "if", "will", "up", "other", "about",
        "out", "many", "then", "them", "these", "so", "some", "would", "make",
        "like", "into", "has", "look", "two", "more", "write",
        "go", "see", "number", "no", "way", "could", "people", "my", "than",
        "first", "water", "been", "call", "who", "oil", "its", "now", "find",
        "long", "down", "day", "did", "get", "come", "made", "may", "part"
    ]);

    /// Words with predefined emojis
    pub static ref PREDEFINED_WORDS: HashMap<&'static str, &'static str> = HashMap::from([
        ("they", "💁"),

    ]);

    /// Some emojis for ALL CAPS SEQUENCES
    pub static ref REPLACEMENT_EMOJIS: [&'static str; 4] = [
        "💀",
        "❗",
        "🔥",
        "👍",
    ];
}
