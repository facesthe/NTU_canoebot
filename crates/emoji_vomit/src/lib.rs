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
mod uwuify;

use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use consts::REPLACEMENT_EMOJIS;
use ntu_canoebot_util::debug_println;

pub use uwuify::uwuify;

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

        let hash_val = rc_hash(&string);
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

/// This iterator iterates over both the words and
/// whitespaces between them.
///
/// A word is defined as a continuous sequence of non-whitespace chars.
///
/// A whitespace sequence is defined as a continuous sequence of whitespace chars.
struct WordSpaceIterator<'a> {
    inner: &'a str,
    index: usize,
    tag: PhantomData<&'a usize>,
}

impl<'a> From<&'a str> for WordSpaceIterator<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: value.trim(),
            index: 0,
            tag: Default::default(),
        }
    }
}

impl<'a> Iterator for WordSpaceIterator<'a> {
    // type Item = WordSpace<'a>; // word-whitespace tuple
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.len() {
            return None;
        }

        let curr_slice = &self.inner[self.index..];

        let mut word_end_idx: usize = self.index;
        let mut iter = curr_slice.chars();

        // searching for word
        // will break on space char
        let space_size = loop {
            let c = iter.next();

            let curr_char = if let Some(_c) = c {
                _c
            } else {
                break 0;
            };

            if curr_char.is_whitespace() {
                break curr_char.len_utf8();
            } else {
                word_end_idx += curr_char.len_utf8();
                continue;
            }
        };

        // debug_println!("word: {} - {}", self.index, word_end_idx);

        let word = &self.inner[self.index..word_end_idx];
        let mut space_end_idx: usize = word_end_idx + space_size;
        self.index = word_end_idx;

        // searching for space
        loop {
            let c = iter.next();

            let curr_char = if let Some(_c) = c {
                _c
            } else {
                break;
            };

            // debug_print!(
            //     "space char: {}, len: {} ",
            //     curr_char.escape_unicode(),
            //     curr_char.len_utf8()
            // );

            if curr_char.is_whitespace() {
                space_end_idx += curr_char.len_utf8();
                continue;
            } else {
                break;
            }
        }

        // debug_println!("space: {} - {}", word_end_idx, space_end_idx);

        let space = &self.inner[word_end_idx..space_end_idx];
        self.index = space_end_idx;

        Some((word, space))
    }
}

/// An iterator over the hashes for a sliding window
/// of strings.
///
/// This is deterministic. Two slices that contain identically hashable
/// elements in the same order will return the same hash iterator.
///
/// A change in the individual hash of an element in the slice will
/// change the sliding window hashes for all elements in the slice.
///
/// The underlying hash method is [rustc_hash], the hashing algorithim used
/// when compiling rust code.
pub struct SlidingWindowHashIterator<'a, T> {
    idx: usize,
    inner: &'a [T],
    window_size: usize,
    window_stop: usize,
}

impl<'a, T: Hash> From<&'a [T]> for SlidingWindowHashIterator<'a, T> {
    fn from(value: &'a [T]) -> Self {
        let all_hash = rc_hash(&value);
        let window_size = all_hash as usize % value.len();
        Self {
            idx: 0,
            inner: value,
            window_size,
            window_stop: value.len() - window_size,
        }
    }
}

impl<'a, T: Hash> Iterator for SlidingWindowHashIterator<'a, T> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.inner.len() {
            return None;
        }

        // forward window
        let window = match self.idx < self.window_stop {
            // sliding window of size self.window_size
            true => &self.inner[self.idx..self.idx + self.window_size],
            // sliding window from idx to last element
            false => &self.inner[self.idx..],
        };

        // reverse window
        let rev_window = match self.idx > self.window_size {
            true => &self.inner[self.idx - self.window_size + 1..=self.idx],
            false => &self.inner[..self.idx],
        };

        let window_hash = rc_hash((window, rev_window));
        self.idx += 1;

        Some(window_hash)
    }
}

/// Create ✨ vomit ✨
pub fn vomit<T: AsRef<str>>(input: T) -> String {
    let slice = input.as_ref();

    let iter = WordSpaceIterator::from(slice);

    let (mut words, spaces): (Vec<Word>, Vec<&str>) = iter.map(|(w, s)| (Word::from(w), s)).unzip();

    // some more processing done here
    assign_emojis(&mut words);
    mod_capitalized_sequence(&mut words);

    words
        .into_iter()
        .zip(spaces.into_iter())
        .map(|(w, s)| format!("{}{}", w.to_string(), s))
        .collect::<String>()
}

/// Look for a matching emoji for a given input.
///
/// Provide a hash value to vary the search result for fuzzy searches.
pub fn find_emoji(word: &str, exact: bool, hash: Option<u64>) -> Option<&'static str> {
    const MASK: usize = 0b111;

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
                let idx = hash as usize & MASK;
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
                let idx = hash as usize & MASK;
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
    let all_hash = rc_hash(&words);
    let window_size = all_hash as usize % words.len();

    debug_println!("window size: {}", window_size);

    let iter = SlidingWindowHashIterator::from(words.as_slice());
    let assigned_emojis: Vec<Option<&'static str>> = words
        .iter()
        .zip(iter)
        .map(|(word, hash)| find_emoji(&word.plaintext(), false, Some(hash)))
        .collect();

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
    let hash_value = rc_hash(&words);
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
fn rc_hash<T: std::hash::Hash>(item: T) -> u64 {
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
    use std::slice::Windows;

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

    #[test]
    fn test_word_space_iterator() {
        let string = "Japan\u{a0} (Japanese: 日本, [ɲihoɴ] , Nippon or Nihon, and formally 日本国, Nippon-koku or Nihon-koku)";
        let string = "To be fair, you have to have a very high IQ to understand Rick and Morty. The humour is extremely subtle, and without a solid grasp of theoretical physics most of the jokes will go over a typical viewer’s head. There’s also Rick’s nihilistic outlook, which is deftly woven into his characterisation- his personal philosophy draws heavily from Narodnaya Volya literature, for instance. The fans understand this stuff; they have the intellectual capacity to truly appreciate the depths of these jokes, to realise that they’re not just funny—they say something deep about LIFE. As a consequence people who dislike Rick & Morty truly ARE idiots- of course they wouldn’t appreciate, for instance, the humour in Rick’s existential catchphrase “Wubba Lubba Dub Dub,” which itself is a cryptic reference to Turgenev’s Russian epic Fathers and Sons. I’m smirking right now just imagining one of those addlepated simpletons scratching their heads in confusion as Dan Harmon’s genius wit unfolds itself on their television screens.";
        let string = "مرحبا, ہیلو, 🌍\u{2002}🌏\u{2003}🌎";

        let iter = WordSpaceIterator::from(string);

        let split: Vec<(&str, &str)> = iter.collect();
        for pair in &split {
            println!(
                "word: \"{}\", space: \"{}\"",
                pair.0,
                pair.1.escape_unicode()
            );
        }
        let reconstructed: String = split.iter().map(|(a, b)| format!("{}{}", a, b)).collect();

        assert_eq!(reconstructed, string);
    }

    /// Testing the correctness of the hash iterator
    #[test]
    fn test_sliding_window_hash_iterator() {
        let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let mut iter = SlidingWindowHashIterator::from(arr.as_slice());

        let window_size = rc_hash(&arr) as usize % arr.len();

        let forward_slice = &arr[..window_size];
        let rev_slice = &arr[0..1];

        let window_hash = rc_hash((forward_slice, rev_slice));
        println!("window size: {}, first hash: {}", window_size, window_hash);

        assert!(matches!(iter.next(), Some(window_hash)));

        // iter completes properly
        for _ in 1..arr.len() {
            iter.next().unwrap();
        }

        assert!(matches!(iter.next(), None))
    }
}
