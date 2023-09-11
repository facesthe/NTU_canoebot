//! Emoji vomit!
//! Given an input string, vomit out some copypasta worthy trash.
//!
//! ## Vomit rules:
//!
//! ### Emoji search
//! A matching emoji is found by cascading through various
//! criteria and matching on the first successful one. They are:
//! - direct name match
//! - annotation
//! - name
//! - text-to-speech
//!
//! ### Ordering
//! - a sequence of uppercase words will have one emoji selected and used at the start and end
//! - some words may also have their corresponding emoji at the start and end. the method used for this is still up for contention
//!

mod consts;

use std::hash::Hasher;

use consts::REPLACEMENT_EMOJIS;
use ntu_canoebot_util::debug_println;

/// UTF-8 string, with additional metadata
#[derive(Clone, Debug, Hash)]
struct Word {
    /// The word within
    data: String,

    /// Marks if the enclosed string does not contain identifiable letters
    has_punctuation: bool,

    /// Marks if every letter of the word is un uppercase
    is_capitalized: bool,

    /// Marks if the emoji is placed before or after the word.
    /// Defaults to false
    pre_fix: bool,

    /// Marks if the emoji is placed before or after the word.
    /// Defaults to true
    post_fix: bool,

    /// Controls the number of times the emoji is repeated
    repeat: u8,

    /// Emoji associated with this word, if any
    emoji: Option<&'static str>,
}

impl<T> From<T> for Word
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let string = value.as_ref().to_string();
        let caps = string
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_punctuation());
        let alphabet = string.chars().all(|c| c.is_alphabetic());
        // let emoji = find_emoji(&string.to_lowercase(), false, None);

        let hash_val = rustc_hash(&string);
        let repeat: u8 = {
            let two = {
                if (hash_val >> 4) & 0b1111 == 0b1111 {
                    // 1 in 8
                    1
                } else {
                    0
                }
            };
            let three = {
                if (hash_val >> 10) & 0b11111 == 0b11111 {
                    // 1 in 16
                    1
                } else {
                    0
                }
            };

            1 + two + three
        };

        Self {
            data: string,
            has_punctuation: !alphabet,
            is_capitalized: caps,
            pre_fix: false,
            post_fix: true,
            repeat,
            emoji: None,
            // emoji,
        }
    }
}

impl ToString for Word {
    fn to_string(&self) -> String {
        let emote = self.emoji.unwrap_or("").to_string();

        let mut res = String::new();
        if self.pre_fix {
            let mut left = self.repeat;
            while left > 0 {
                res += &emote;
                left -= 1;
            }
        }

        res += &self.data;

        if self.post_fix {
            let mut left = self.repeat;
            while left > 0 {
                res += &emote;
                left -= 1;
            }
        }

        res
    }
}

impl Word {
    /// Returns the contents of the word in lowercase
    /// and without punctuation.
    pub fn plaintext(&self) -> String {
        let inter = self.data.to_lowercase();

        if self.has_punctuation {
            remove_ascii_punctuation(inter)
        } else {
            inter
        }
    }
}

/// Create ✨ vomit ✨
pub fn vomit<T: AsRef<str>>(input: T) -> String {
    let slice = input.as_ref();

    let mut words: Vec<Word> = slice
        .split_whitespace()
        .map(|word| Word::from(word))
        .collect();

    // some more processing done here
    assign_emojis(&mut words);
    mod_capitalized_sequence(&mut words);

    words
        .into_iter()
        .map(|w| w.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Look for a matching emoji for a given input.
///
/// Provide a hash value to vary the search result for fuzzy searches.
pub fn find_emoji(word: &str, exact: bool, hash: Option<u64>) -> Option<&'static str> {
    // ignore words in this set
    if consts::IGNORE_WORDS.contains(word) {
        return None;
    }
    if consts::PREDEFINED_WORDS.contains_key(word) {
        return consts::PREDEFINED_WORDS.get(word).cloned();
    }

    // this branch is taken very rarely
    if let Some(e) = emoji::lookup_by_name::lookup(word) {
        // debug_println!("direct match: \"{}\" = {:#?}", word, e);
        return Some(e.glyph);
    }

    if exact {
        return None;
    }

    // annotation search
    let matching_annotations = emoji::search::search_annotation_all(word);
    match matching_annotations.len() {
        0 => (),
        _ => {
            let e = if let Some(hash) = hash {
                let idx = hash as usize & 0b111;
                match matching_annotations.get(idx) {
                    Some(emote) => emote,
                    None => matching_annotations.first().unwrap(),
                }
            } else {
                matching_annotations.first().unwrap()
                // debug_println!("annotation match: \"{}\" = {:#?}", word, e);
            };

            return Some(e.glyph);
        }
    }
    // name search
    let matching_names = emoji::search::search_name(word);
    // debug_println!("num matching: {}", matching_names.len());

    match matching_names.len() {
        0 => (),
        _ => {
            let e = if let Some(hash) = hash {
                let idx = hash as usize & 0b11;
                match matching_names.get(idx) {
                    Some(emote) => emote,
                    None => matching_names.first().unwrap(),
                }
            } else {
                matching_names.first().unwrap()
            };

            // debug_println!("name partial match: \"{}\" = {:#?}", word, e);
            return Some(e.glyph);
        }
    }

    // tts
    let matching_tts = emoji::search::search_tts_all(word);
    match matching_tts.len() {
        0 => (),
        _ => {
            let e = matching_tts.first().unwrap();
            debug_println!("tts match: \"{}\" = {:#?}", word, e);
            return Some(e.glyph);
        }
    }

    None
}

/// Capitalized sequences have their emojis appear at the start and
/// end. An emoji is selected based on the hash value of these words,
/// for repeatability.
///
/// Example:
/// "this is SO FIRE" where word FIRE => '🔥'
/// turns into something like:
///
/// "this is 🔥SO FIRE🔥"
///
/// A deterministic hashing algorithm ([rustc_hash]) is used
/// to give repeatable results, while still providing some emoji
/// variations between different text sequences.
fn mod_capitalized_sequence(words: &mut Vec<Word>) {
    // index markers
    let mut mark_start: Option<usize> = None;
    let mut mark_end: Option<usize> = None;

    for idx in 0..words.len() {
        let word = &words[idx];

        if word.is_capitalized {
            match (mark_start, mark_end) {
                (None, _) => {
                    mark_start = Some(idx);
                    mark_end = Some(idx);
                }
                (Some(_), _) => {
                    mark_end = Some(idx);
                }
            }

            // count the sequence as complete for capitalized words that
            // contain ASCII punctuation
            match (mark_start, mark_end) {
                (Some(start), Some(end)) => {
                    if word.has_punctuation {
                        mut_caps_vec(&mut words[start..=end]);
                        mark_start = None;
                        mark_end = None;
                    }
                }
                _ => (),
            }
        } else {
            match (mark_start, mark_end) {
                (Some(start), Some(end)) => {
                    // skip single words w/ single letter
                    if start == end && words[start].data.len() == 1 {
                        continue;
                    }

                    // debug_println!("modifying sequence from idx {} - idx {}", start, end);
                    mut_caps_vec(&mut words[start..=end]);
                    mark_start = None;
                    mark_end = None;
                }
                _ => (),
            }
        }
    }

    // for the case when the last element is all caps
    match (mark_start, mark_end) {
        (Some(start), Some(end)) => mut_caps_vec(&mut words[start..=end]),
        _ => (),
    }
}

/// Find and assign emojis according to the hash of a sliding window of words.
///
/// This process is fully deterministic: the same vector of [Word] will yield the same
/// result.
fn assign_emojis(words: &mut Vec<Word>) {
    let all_hash = rustc_hash(&words);
    let window_size = all_hash as usize % words.len();

    debug_println!("window size: {}", window_size);

    let mut assigned_emojis: Vec<Option<&'static str>> = match window_size {
        0 => Vec::new(),
        _ => words
            .windows(window_size)
            .map(|window| {
                let window_hash = rustc_hash(&window);
                let emoji = find_emoji(&window[0].plaintext(), false, Some(window_hash));

                emoji
            })
            .collect(),
    };

    // this idx all the way to (1 - words.len()) are yet to be hashed
    // the windows for these will be from the current word to the last word
    // in the vector.
    let size = words.len();
    let idx_remaining = size - window_size + 1;

    if idx_remaining < size {
        for idx in (idx_remaining..size).into_iter() {
            let window = &words[idx..size];
            let word = &words[idx];
            let hash = rustc_hash(&window);

            let emoji = find_emoji(&word.plaintext(), false, Some(hash));
            assigned_emojis.push(emoji);
        }
    }

    debug_println!(
        "word size: {}, emoji size: {}",
        words.len(),
        assigned_emojis.len()
    );

    for (word, assigned) in words.iter_mut().zip(assigned_emojis.into_iter()) {
        word.emoji = assigned;
    }
}

/// Perform mutation for a slice of known capitalized words.
/// The emoji taken is based off the hash of all words
fn mut_caps_vec(words: &mut [Word]) {
    let hash_value = rustc_hash(&words);
    let sequence_len = words.len();

    // the emoji's index we want to take
    let idx: usize = hash_value as usize % sequence_len;
    let mut emoji_taken =
        words[idx..]
            .iter()
            .find_map(|w| if let Some(e) = w.emoji { Some(e) } else { None });

    // search the rest of sequence if emoji not found
    if let None = emoji_taken {
        emoji_taken = words
            .iter()
            .find_map(|w| if let Some(e) = w.emoji { Some(e) } else { None });
    }

    // if there is still no emoji, use the exclamation mark emoji
    if let None = emoji_taken {
        let new_idx = hash_value as usize % REPLACEMENT_EMOJIS.len();
        let replacement = REPLACEMENT_EMOJIS[new_idx];
        emoji_taken = Some(replacement)
    }

    // debug_println!("emoji found: {:?}", emoji_taken);

    for word in words.iter_mut() {
        word.emoji = None;
        word.pre_fix = false;
        word.post_fix = false;
    }

    if let Some(f) = words.first_mut() {
        f.emoji = emoji_taken;
        f.pre_fix = true;
    }

    if let Some(f) = words.last_mut() {
        f.emoji = emoji_taken;
        f.post_fix = true;
    }
}

/// I don't know why this isnt in the standard library.
///
/// Uses [rustc_hash] for repeatable deterministic hashing.
fn rustc_hash<T: std::hash::Hash>(item: T) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    item.hash(&mut hasher);
    hasher.finish()
}

fn remove_ascii_punctuation<T: AsRef<str>>(slice: T) -> String {
    slice
        .as_ref()
        .chars()
        .filter_map(|c| {
            if c.is_ascii_punctuation() {
                None
            } else {
                Some(c)
            }
        })
        .collect()
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn test_for_sequence() {
        // println!("{:?}", matches);

        let pasta_a = "What the fuck did you just fucking say about me, you little bitch? I'll have you know I graduated top of my class in the Navy Seals, and I've been involved in numerous secret raids on Al-Quaeda, and I have over 300 confirmed kills. I am trained in gorilla warfare and I'm the top sniper in the entire US armed forces. You are nothing to me but just another target. I will wipe you the fuck out with precision the likes of which has never been seen before on this Earth, mark my fucking words. You think you can get away with saying that shit to me over the Internet? Think again, fucker. As we speak I am contacting my secret network of spies across the USA and your IP is being traced right now so you better prepare for the storm, maggot. The storm that wipes out the pathetic little thing you call your life. You're fucking dead, kid. I can be anywhere, anytime, and I can kill you in over seven hundred ways, and that's just with my bare hands. Not only am I extensively trained in unarmed combat, but I have access to the entire arsenal of the United States Marine Corps and I will use it to its full extent to wipe your miserable ass off the face of the continent, you little shit. If only you could have known what unholy retribution your little 'clever' comment was about to bring down upon you, maybe you would have held your fucking tongue. But you couldn't, you didn't, and now you're paying the price, you goddamn idiot. I will shit fury all over you and you will drown in it. You're fucking dead, kiddo.";

        let res = vomit(pasta_a);

        println!("{}", res);

        let pasta_b = " So the other day, I was playing rainbow six siege, and I heard one of my teammates make a callout in the voice chat. It was a real life gamer girl. God, I kid you not, I just stopped playing and pulled my dick out. “fuck, Fuck!” I was yelling in voice chat. I just wanted to hear her voice again. “Please,” I moaned. But she left the lobby. I was crying and covered in my own cum, but I remembered that I could find recent teammates in the ubiplay friends tab. I frantically closed down siege and opened the tab, to find out she had TTV IN HER NAME!!! She was streaming, and only had 100 viewers!!! The competition was low, so I made the first move and donated my months rent to her. I was already about to pre. She read my donation in the chat. God this is the happiest I’ve been in a long time. I did a little research, and found out where she goes to school, but I am a little nervous to talk to her in person, and need support. Any advice before my Uber gets to her middle school?";

        let res = vomit(pasta_b);
        println!("{}", res);

        let pasta_c = "OH MY GOD WHAT HAPPENED TO YOUR MOM man";
        let res = vomit(pasta_c);
        println!("{}", res);
    }

    #[test]
    fn test_common_words() {
        const TOP: [&str; 99] = [
            "the", "of", "and", "to", "in", "a", "is", "that", "it", "he", "was", "for", "on",
            "are", "as", "with", "his", "they", "I", "at", "be", "this", "have", "from", "or",
            "one", "had", "by", "word", "but", "not", "what", "all", "were", "we", "when", "your",
            "can", "said", "there", "use", "an", "each", "which", "she", "do", "how", "their",
            "if", "will", "up", "other", "about", "out", "many", "then", "them", "these", "so",
            "some", "her", "would", "make", "like", "him", "into", "time", "has", "look", "two",
            "more", "write", "go", "see", "number", "no", "way", "could", "people", "my", "than",
            "first", "water", "been", "call", "who", "oil", "its", "now", "find", "long", "down",
            "day", "did", "get", "come", "made", "may", "part",
        ];

        for t in TOP {
            let res = find_emoji(t, false, None);

            println!("\"{}\" => {}", t, res.unwrap_or_default())
        }
    }

    #[test]
    fn test_word() {
        let pasta_c = "OH MY GOD WHAT HAPPENED hahahahaha";
        let mut words: Vec<Word> = pasta_c.split_whitespace().map(|w| Word::from(w)).collect();

        println!("{:#?}", words);

        let res = vomit(pasta_c);
        println!("{}", res);

        mut_caps_vec(&mut words);
        println!("{:?}", words);
    }

    #[test]
    fn test_strip_punctuation() {
        let x = "asd!!!!!".to_string();
        let x: String = x
            .chars()
            .filter_map(|c| {
                if c.is_ascii_punctuation() {
                    None
                } else {
                    Some(c)
                }
            })
            .collect();

        println!("{}", x);
    }
}
