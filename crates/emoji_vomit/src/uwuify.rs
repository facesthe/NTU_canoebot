//! Another useless string transformer.
//!
//! Turns normal text to something a weeb might type
#![allow(unused)]

use std::collections::HashSet;

use lazy_static::lazy_static;
use ntu_canoebot_util::{debug_print, debug_println};

use crate::{rc_hash, SlidingWindowHashIterator, WordSpaceIterator};

lazy_static! {
    /// chars to switch to 'w'
    pub static ref CHARS_TO_BE_SWITCHED: HashSet<char> = HashSet::from(['l', 'r']);

    /// Normal-looking text faces
    pub static ref KAOMOJI_NORMAL: Vec<&'static str> = vec![
        "(* ^ ω ^)",
        "<(￣︶￣)>",
        "(⌒▽⌒)☆",
        "(¬‿¬ )",
        "(￣～￣;)",
        "(￢ ￢)",

    ];

    /// Extreme-looking text faces
    pub static ref KAOMOJI_EXTREME: Vec<&'static str> = vec![
        "(；￣Д￣)",
        "(；⌣̀_⌣́)",
        "凸(￣ヘ￣)",
        "٩(╬ʘ益ʘ╬)۶",
        "(>_<)",
        "(;;;*_*)",
    ];

    /// Stuff that does not look like a face
    pub static ref KAOMOJI_NOT_FACE: Vec<&'static str> = vec![
        "*:･ﾟ’★,｡･:*:･ﾟ’☆",

    ];

}

/// Create UwU
pub fn uwuify<T: AsRef<str>>(input: T) -> String {
    let slice = input.as_ref();

    let (mut words, spaces): (Vec<String>, Vec<&str>) = WordSpaceIterator::from(slice)
        .map(|(w, s)| (w.to_owned(), s))
        .unzip();

    let hash_vec = SlidingWindowHashIterator::from(words.as_slice()).collect::<Vec<_>>();

    // basic uwu conversion
    w_conversion(&mut words);
    // dashes
    add_dashes(&mut words, &hash_vec);
    // faces
    add_kaomoji(&mut words, &hash_vec);

    let res = words
        .into_iter()
        .zip(spaces.iter())
        .map(|(w, s)| w + s)
        .collect::<String>();

    res
}

/// Turn some characters to 'w'
fn w_conversion(sequence: &mut Vec<String>) {
    for word in sequence {
        let changed = word
            .chars()
            .map(|c| {
                if CHARS_TO_BE_SWITCHED.contains(&c) {
                    'w'
                } else {
                    c
                }
            })
            .collect::<String>();

        if *word != changed {
            *word = changed
        }
    }
}

/// Adds dashes to a word depending on a hash
/// of a sliding window of words
///
/// For example: "wow" may get modified to "w-wow"
fn add_dashes(sequence: &mut Vec<String>, hashes: &[u64]) {
    let modified = sequence
        .iter()
        .zip(hashes.iter())
        .map(|(word, hash)| {
            let filt = *hash as usize & binary_ones(word.len());

            if filt != 0b0 {
                return word.to_owned();
            }

            let first_char = {
                let res = word.chars().next();
                match res {
                    Some(c) => c,
                    None => return word.to_owned(),
                }
            };

            format!("{}-{}", first_char, word)
        })
        .collect::<Vec<_>>();

    *sequence = modified
}

/// Adds text faces
fn add_kaomoji(sequence: &mut Vec<String>, hashes: &[u64]) {

    // todo!()
}

/// Returns the minumum number of bits required to represent
/// a particular number
fn min_bits(mut num: usize) -> usize {
    let mut count: usize = 0;

    loop {
        if num != 0 {
            num >>= 1;
            count += 1;
        } else {
            break;
        }
    }

    count
}

/// Constructs a number with a binary representation of
/// all ones.
fn binary_ones(len: usize) -> usize {
    let mut start: usize = 0;

    for _ in 0..len {
        start <<= 1;
        start += 1;
    }

    start
}

#[cfg(test)]
mod tests {

    use super::*;

    const STRING_A: &'static str = "What the fuck did you just fucking say about me, you little bitch? I'll have you know I graduated top of my class in the Navy Seals, and I've been involved in numerous secret raids on Al-Quaeda, and I have over 300 confirmed kills. I am trained in gorilla warfare and I'm the top sniper in the entire US armed forces. You are nothing to me but just another target. I will wipe you the fuck out with precision the likes of which has never been seen before on this Earth, mark my fucking words. You think you can get away with saying that shit to me over the Internet? Think again, fucker. As we speak I am contacting my secret network of spies across the USA and your IP is being traced right now so you better prepare for the storm, maggot. The storm that wipes out the pathetic little thing you call your life. You're fucking dead, kid. I can be anywhere, anytime, and I can kill you in over seven hundred ways, and that's just with my bare hands. Not only am I extensively trained in unarmed combat, but I have access to the entire arsenal of the United States Marine Corps and I will use it to its full extent to wipe your miserable ass off the face of the continent, you little shit. If only you could have known what unholy retribution your little 'clever' comment was about to bring down upon you, maybe you would have held your fucking tongue. But you couldn't, you didn't, and now you're paying the price, you goddamn idiot. I will shit fury all over you and you will drown in it. You're fucking dead, kiddo.";
    const STRING_B: &'static str = "To be fair, you have to have a very high IQ to understand Rick and Morty. The humour is extremely subtle, and without a solid grasp of theoretical physics most of the jokes will go over a typical viewer’s head. There’s also Rick’s nihilistic outlook, which is deftly woven into his characterisation- his personal philosophy draws heavily from Narodnaya Volya literature, for instance. The fans understand this stuff; they have the intellectual capacity to truly appreciate the depths of these jokes, to realise that they’re not just funny—they say something deep about LIFE. As a consequence people who dislike Rick & Morty truly ARE idiots- of course they wouldn’t appreciate, for instance, the humour in Rick’s existential catchphrase “Wubba Lubba Dub Dub,” which itself is a cryptic reference to Turgenev’s Russian epic Fathers and Sons. I’m smirking right now just imagining one of those addlepated simpletons scratching their heads in confusion as Dan Harmon’s genius wit unfolds itself on their television screens.";

    #[test]
    fn test_uwuify() {
        let string = STRING_A;
        let string = STRING_B;

        let res = uwuify(string);

        println!("{}", res);
    }

    #[test]
    fn test_bit_count() {
        assert_eq!(min_bits(0), 0);
        assert_eq!(min_bits(1), 1);
        assert_eq!(min_bits(2), 2);
        assert_eq!(min_bits(4), 3);
        assert_eq!(min_bits(8), 4);
        assert_eq!(min_bits(9), 4);
    }

    #[test]
    fn test_binary_ones() {
        assert_eq!(binary_ones(0), 0b0);
        assert_eq!(binary_ones(1), 0b1);
        assert_eq!(binary_ones(2), 0b11);
        assert_eq!(binary_ones(3), 0b111);
        assert_eq!(binary_ones(4), 0b1111);
        assert_eq!(binary_ones(5), 0b11111);
    }
}
