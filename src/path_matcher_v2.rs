// pub mod path-matcher
use std::collections::HashMap;
use std::error::Error;
use std::ops::Index;


use regex::{Regex, Captures};
use substring::Substring;


static DEFAULT_PATH_SEPARATOR: &'static str = "/";

static CACHE_TURNOFF_THRESHOLD: i32 = 65536;

static VARIABLE_PATTERN: &'static str = "\\{[^/]+?\\}";

static WILDCARD_CHARS: [char; 3] = ['*', '?', '{'];

static GLOB_PATTERN: &'static str = r"\?|\*|\{((?:\{[^/]+?}|[^/{}]|\\[{}])+?)}";

static DEFAULT_VARIABLE_PATTERN: &'static str = "(.*)";

pub trait PathMatcher {
    fn is_pattern(&self, path: &str) -> bool;

    fn match_(&mut self, pattern: &str, path: &str) -> bool;

    fn match_start(&self, pattern: String, path: String) -> bool;
    fn extract_path_within_pattern(&mut self, pattern: &str, path: &str) -> String;

    fn extract_uri_template_variables(&mut self, pattern: &str, path: &str) -> HashMap<String, String>;

    fn combine(&self, pattern1: String, pattern2: String) -> String;
}


pub struct AntPathMatcher {
    path_separator: String,
    path_separator_pattern_cache: PathSeparatorPatternCache,
    case_sensitive: bool,
    trim_tokens: bool,
    cache_patterns: bool,
    tokenized_pattern_cache: HashMap<String, Vec<String>>,
    string_matcher_cache: HashMap<String, AntPathStringMatcher>,
}


impl AntPathMatcher {}

impl Default for AntPathMatcher {
    fn default() -> Self {
        AntPathMatcher {
            path_separator: DEFAULT_PATH_SEPARATOR.to_string(),
            path_separator_pattern_cache: PathSeparatorPatternCache::new(DEFAULT_PATH_SEPARATOR),
            case_sensitive: false,
            trim_tokens: false,
            cache_patterns: false,
            tokenized_pattern_cache: HashMap::default(),
            string_matcher_cache: HashMap::default(),
        }
    }
}


impl PathMatcher for AntPathMatcher {
    fn is_pattern(&self, path: &str) -> bool {
        todo!()
    }

    fn match_(&mut self, pattern: &str, path: &str) -> bool {
        todo!()
    }

    fn match_start(&self, pattern: String, path: String) -> bool {
        todo!()
    }

    fn extract_path_within_pattern(&mut self, pattern: &str, path: &str) -> String {
        todo!()
    }

    fn extract_uri_template_variables(&mut self, pattern: &str, path: &str) -> HashMap<String, String> {
        todo!()
    }

    fn combine(&self, pattern1: String, pattern2: String) -> String {
        todo!()
    }
}


#[derive(Clone)]
struct AntPathStringMatcher {
    raw_patten: String,
    exact_match: bool,

    case_sensitive: bool,
    regex: Regex,
    variable_names: Vec<String>,
}

impl AntPathStringMatcher {
    fn new(patten: &str, case_sensitive: bool) {
        let mut pattern_builder = String::new();
        let mut variable_names: Vec<&str> = Vec::new();
        let end: usize = 0;
        for (i, c) in Regex::new(GLOB_PATTERN).unwrap().captures_iter(&patten).enumerate() {
            pattern_builder.push_str(quote(patten, end, c.get(0).unwrap().start()).as_str());
            let match_ = &c[0];

            if "?" == match_ {
                pattern_builder.push_str(".");
            } else if "*" == match_ {
                pattern_builder.push_str(".*");
            } else if match_.starts_with("{") && match_.ends_with("}") {
                let colon_idx = match_.find(':');
                if colon_idx.is_none() {
                    pattern_builder.push_str(DEFAULT_VARIABLE_PATTERN);
                    // variable_names.push(&c[1])
                } else {
                    // String variable_pattern = match.substring(colonIdx + 1, match.length() - 1);
                    // pattern_builder.append('(');
                    // pattern_builder.append(variable_pattern);
                    // pattern_builder.append(')');
                    // String variableName = match.substring(1, colonIdx);
                    // this.variableNames.add(variableName);
                    let variable_pattern = match_.substring(colon_idx? + 1, match_.len() - 1);
                    pattern_builder.push_str("(");
                    pattern_builder.push_str(variable_pattern);
                    pattern_builder.push_str(")");
                    let variable_name = match_.substring(1, colon_idx?);

                    variable_names.push(variable_name);
                }
            }


            // for j in 0..c.len() {
            //     &c[0]
            // }
        }
        todo!()
    }
}


// TODO 不知道是否等价
pub fn quote(s: &str, start: usize, end: usize) -> String {
    if start == end {
        return "".to_string();
    }
    return regex::escape(s.substring(start, end));
}

struct PathSeparatorPatternCache {
    pub ends_on_wildcard: String,
    pub ends_on_double_wildcard: String,
}

impl PathSeparatorPatternCache {
    fn new(path_separator: &str) -> Self {
        let mut ends_on_wildcard = path_separator.to_string();
        let mut ends_on_double_wildcard = path_separator.to_string();
        ends_on_wildcard.push_str("*");
        ends_on_double_wildcard.push_str("**");

        PathSeparatorPatternCache {
            ends_on_wildcard,
            ends_on_double_wildcard,
        }
    }
}

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    use regex::{Regex, Captures};
    let s = "123-4567-89,987-6543-21";
    let r = Regex::new(r"\d{3}-(\d{4})-\d{2}")?;
    if r.is_match(s) { // if let m = r.find(s) {
        println!("Found Matches:")
    }

    for (i, c) in r.captures_iter(&s).enumerate() {
        for j in 0..c.len() {
            println!("group {},{} : {}", i, j, &c[j]);
        }
    }

    let r2 = Regex::new(r"(\d+)-(\d+)-(\d+)")?;
    let s2 = r2.replace_all(&s, "$3-$1-$2");
    println!("{}", s2);

    let r3 = Regex::new(r"\d+")?;
    let s3 = r3.replace_all(&s, |c: &Captures| c[0].chars().rev().collect::<String>());
    println!("{}", s3);

    let r4 = Regex::new("%(begin|next|end)%")?;
    let s4 = "%begin%hello%next%world%end%";
    // let v = r4.split(s4).collect_vec();
    // println!("{:?}", v);


    Ok(())
}

#[test]
fn str() {
    use substring::Substring;
    let match_ = "hamburger";
    println!("{}", match_.substring(4, match_.len()));
}