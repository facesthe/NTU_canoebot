//! Emoji vomit!
//! Given an input string, vomit out some copypasta worthy trash.
//!
//! ## Vomit rules:
//!
//!

mod consts;

use ntu_canoebot_util::debug_println;

/// Create ✨ vomit ✨
pub fn vomit<T: AsRef<str>>(input: T) -> String {
    let slice = input.as_ref();

    let res = slice
        .split_whitespace()
        .into_iter()
        .map(|word| {
            // word.split_ascii_whitespace(

            let res = find_emoji(word, false);

            [word, res.unwrap_or("")].concat()
        })
        .collect::<Vec<String>>()
        .join(" ");

    res
}

/// Look for a matching emoji for a given input
pub fn find_emoji(word: &str, exact: bool) -> Option<&str> {
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
            let e = matching_annotations.first().unwrap();
            // debug_println!("annotation match: \"{}\" = {:#?}", word, e);
            return Some(e.glyph);
        }
    }
    // name search
    let matching_names = emoji::search::search_name(word);
    match matching_names.len() {
        0 => (),
        _ => {
            let e = matching_names.first().unwrap();
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

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[test]
    fn test_for_sequence() {
        // println!("{:?}", matches);

        let pasta_a = "What the fuck did you just fucking say about me, you little bitch? I'll have you know I graduated top of my class in the Navy Seals, and I've been involved in numerous secret raids on Al-Quaeda, and I have over 300 confirmed kills. I am trained in gorilla warfare and I'm the top sniper in the entire US armed forces. You are nothing to me but just another target. I will wipe you the fuck out with precision the likes of which has never been seen before on this Earth, mark my fucking words. You think you can get away with saying that shit to me over the Internet? Think again, fucker. As we speak I am contacting my secret network of spies across the USA and your IP is being traced right now so you better prepare for the storm, maggot. The storm that wipes out the pathetic little thing you call your life. You're fucking dead, kid. I can be anywhere, anytime, and I can kill you in over seven hundred ways, and that's just with my bare hands. Not only am I extensively trained in unarmed combat, but I have access to the entire arsenal of the United States Marine Corps and I will use it to its full extent to wipe your miserable ass off the face of the continent, you little shit. If only you could have known what unholy retribution your little 'clever' comment was about to bring down upon you, maybe you would have held your fucking tongue. But you couldn't, you didn't, and now you're paying the price, you goddamn idiot. I will shit fury all over you and you will drown in it. You're fucking dead, kiddo.";

        let res = vomit(pasta_a);

        println!("{}", res)
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
            let res = find_emoji(t, false);

            println!("\"{}\" => {}", t, res.unwrap_or_default())
        }
    }
}
