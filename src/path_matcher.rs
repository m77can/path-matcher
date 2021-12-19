use std::collections::HashMap;

use regex::{Regex, RegexBuilder};
use substring::Substring;

static DEFAULT_PATH_SEPARATOR: &'static str = "/";

static CACHE_TURNOFF_THRESHOLD: usize = 65536;

static WILDCARD_CHARS: [char; 3] = ['*', '?', '{'];

static GLOB_PATTERN: &'static str = r"\?|\*|\{((?:\{[^/]+?\}|[^/{}]|\\[{}])+?)\}";

static DEFAULT_VARIABLE_PATTERN: &'static str = "((?s).*)";

pub trait PathMatcher {
    fn is_pattern(&self, path: &str) -> bool;

    fn match_(&mut self, pattern: &str, path: &str) -> bool;

    fn match_start(&mut self, pattern: &str, path: &str) -> bool;
    fn extract_path_within_pattern(&mut self, pattern: &str, path: &str) -> String;

    fn extract_uri_template_variables(
        &mut self,
        pattern: &str,
        path: &str,
    ) -> HashMap<String, String>;
}

pub struct AntPathMatcher {
    path_separator: String,
    case_sensitive: bool,
    trim_tokens: bool,
    cache_patterns: Option<bool>,
    tokenized_pattern_cache: HashMap<String, Vec<String>>,
    string_matcher_cache: HashMap<String, AntPathStringMatcher>,
}

impl AntPathMatcher {
    pub fn new(
        path_separator: &str,
        case_sensitive: bool,
        trim_tokens: bool,
        cache_patterns: bool,
    ) -> Self {
        AntPathMatcher {
            path_separator: path_separator.to_string(),
            case_sensitive,
            trim_tokens,
            cache_patterns: Some(cache_patterns),
            tokenized_pattern_cache: HashMap::new(),
            string_matcher_cache: HashMap::new(),
        }
    }

    fn do_match(
        &mut self,
        pattern: &str,
        path: &str,
        full_path: bool,
        uri_template_variables: &mut Option<HashMap<String, String>>,
    ) -> bool {
        if path.starts_with(self.path_separator.clone().as_str())
            != pattern.starts_with(self.path_separator.clone().as_str()) {
            return false;
        }

        let patt_dirs = self.tokenize_pattern(pattern);
        if full_path && self.case_sensitive && !self.is_potential_match(path, &patt_dirs) {
            return false;
        }


        let path_dirs: Vec<String> = self.tokenize_path(path);
        let mut patt_idx_start: isize = 0;
        let mut patt_idx_end: isize = patt_dirs.len() as isize - 1;
        let mut path_idx_start: isize = 0;
        let mut path_idx_end: isize = path_dirs.len() as isize - 1;

        // Match all elements up to the first **
        while patt_idx_start <= patt_idx_end && path_idx_start <= path_idx_end {
            let patt_dir = &patt_dirs[patt_idx_start as usize];
            if patt_dir == "**" {
                break;
            }
            if !self.match_strings(patt_dir, &path_dirs[path_idx_start as usize], uri_template_variables) {
                return false;
            }
            patt_idx_start += 1;
            path_idx_start += 1;
        }

        if path_idx_start > path_idx_end {
            // Path is exhausted, only match if rest of pattern is * or **'s
            if patt_idx_start > patt_idx_end {
                return pattern.ends_with(self.path_separator.clone().as_str()) == path.ends_with(self.path_separator.clone().as_str());
            }
            if !full_path {
                return true;
            }
            if patt_idx_start == patt_idx_end && patt_dirs[patt_idx_start as usize] == "*" && path.ends_with(self.path_separator.clone().as_str()) {
                return true;
            }
            for i in patt_idx_start..=patt_idx_end {
                if patt_dirs[i as usize] != "**" {
                    return false;
                }
            }
            return true;
        } else if patt_idx_start > patt_idx_end {
            // String not exhausted, but pattern is. Failure.
            return false;
        } else if !full_path && "**" == patt_dirs[patt_idx_start as usize] {
            // Path start definitely matches due to "**" part in pattern.
            return true;
        }

        // up to last '**'
        while patt_idx_start <= patt_idx_end && path_idx_start <= path_idx_end {
            let patt_dir = &patt_dirs[patt_idx_end as usize];
            if patt_dir == "**" {
                break;
            }
            if !self.match_strings(patt_dir, &path_dirs[path_idx_end as usize], uri_template_variables) {
                return false;
            }
            patt_idx_end -= 1;
            path_idx_end -= 1;
        }

        if path_idx_start > path_idx_end {
            // String is exhausted
            for i in patt_idx_start..=patt_idx_end {
                if patt_dirs[i as usize] != "**" {
                    return false;
                }
            }
            return true;
        }

        while patt_idx_start != patt_idx_end && path_idx_start <= path_idx_end {
            let mut pat_idx_tmp: isize = -1;
            for i in patt_idx_start + 1..=patt_idx_end {
                if patt_dirs[i as usize] == "**" {
                    pat_idx_tmp = i as isize;
                    break;
                }
            }

            if pat_idx_tmp == patt_idx_start as isize + 1 {
                // '**/**' situation, so skip one
                patt_idx_start += 1;
                continue;
            }

            let pat_length = pat_idx_tmp - (patt_idx_start as isize) - 1;
            let str_length: isize = (path_idx_end - path_idx_start + 1) as isize;
            let mut found_idx: isize = -1;

            'out: for i in 0..=str_length - pat_length {
                for j in 0..pat_length {
                    let sub_pat = &patt_dirs[(patt_idx_start + j + 1) as usize];
                    let sub_str = &path_dirs[(path_idx_start + i + j) as usize];
                    if !self.match_strings(sub_pat, sub_str, uri_template_variables) {
                        continue 'out;
                    }
                }
                found_idx = (path_idx_start + i) as isize;
                break;
            }

            if found_idx == -1 {
                return false;
            }
            patt_idx_start = pat_idx_tmp;
            path_idx_start = found_idx + pat_length;
        }

        for i in patt_idx_start..=patt_idx_end {
            if patt_dirs[i as usize] != "**" {
                return false;
            }
        }

        return true;
    }

    fn is_potential_match(&self, path: &str, patt_dirs: &Vec<String>) -> bool {
        if !self.trim_tokens {
            let mut pos: usize = 0;
            for patt_dir in patt_dirs {
                let skipped = self.skip_separator(path, pos, self.path_separator.clone().as_str());
                pos += skipped;
                let skipped = self.skip_segment(path, pos, patt_dir);
                if skipped < patt_dir.len() {
                    return skipped > 0 || (patt_dir.len() > 0 && self.is_wildcard_char(patt_dir.chars().next().unwrap()));
                }
                pos += skipped;
            }
        }
        return true;
    }

    fn skip_segment(&self, patch: &str, pos: usize, prefix: &str) -> usize {
        let mut skipped = 0;
        for c in prefix.chars() {
            if self.is_wildcard_char(c) {
                return skipped;
            }
            let curr_pos = pos + skipped;
            if curr_pos >= patch.len() {
                return 0;
            }
            if c == patch.chars().nth(curr_pos).unwrap() {
                skipped += 1;
            }
        }
        return skipped;
    }

    fn skip_separator(&self, path: &str, pos: usize, separator: &str) -> usize {
        let mut skipped = 0;
        while path[pos + skipped..].starts_with(separator) {
            skipped += separator.len();
        }
        skipped
    }

    fn is_wildcard_char(&self, c: char) -> bool {
        for candidate in WILDCARD_CHARS.iter() {
            if &c == candidate {
                return true;
            }
        }
        return false;
    }

    fn match_strings(
        &mut self,
        pattern: &str,
        str: &str,
         uri_template_variables: &mut Option<HashMap<String, String>>,
    ) -> bool {
        let string_matcher = self.get_string_matcher(pattern);
        string_matcher.match_string(str, uri_template_variables)
    }

    fn get_string_matcher(&mut self, pattern: &str) -> AntPathStringMatcher {
        let mut matcher: Option<AntPathStringMatcher> = None;
        if self.cache_patterns.is_none() || self.cache_patterns.unwrap() {
            matcher = self.string_matcher_cache.get(pattern).cloned();
        }

        if matcher.is_none() {
            matcher = Some(AntPathStringMatcher::new(pattern, self.case_sensitive));

            if self.cache_patterns.is_none()
                && self.string_matcher_cache.len() >= CACHE_TURNOFF_THRESHOLD
            {
                // Try to adapt to the runtime situation that we're encountering:
                // There are obviously too many different patterns coming in here...
                // So let's turn off the cache since the patterns are unlikely to be reoccurring.
                self.deactivate_pattern_cache();
                return matcher.unwrap().clone();
            }
            if self.cache_patterns.is_none() || self.cache_patterns.unwrap() {
                self.string_matcher_cache
                    .insert(pattern.to_string(), matcher.as_ref().unwrap().clone());
            }
        }
        matcher.unwrap().clone()
    }

    fn tokenize_pattern(&mut self, pattern: &str) -> Vec<String> {
        let mut tokenized: Vec<String> = Vec::new();
        let cache_patterns = self.cache_patterns;

        if cache_patterns.is_none() || cache_patterns.unwrap() {
            match self.tokenized_pattern_cache.get(pattern) {
                Some(tokenized_pattern) => {
                    tokenized = tokenized_pattern.clone();
                }
                None => {
                    tokenized = self.tokenize_path(pattern);

                    if cache_patterns.is_none()
                        && self.tokenized_pattern_cache.len() >= CACHE_TURNOFF_THRESHOLD
                    {
                        // Try to adapt to the runtime situation that we're encountering:
                        // There are obviously too many different patterns coming in here...
                        // So let's turn off the cache since the patterns are unlikely to be reoccurring.
                        self.deactivate_pattern_cache();
                        return tokenized;
                    }
                    if cache_patterns.is_none() || cache_patterns.unwrap() {
                        self.tokenized_pattern_cache
                            .insert(pattern.to_string(), tokenized.clone());
                    }
                }
            }
        }
        return tokenized;
    }

    fn tokenize_path(&self, path: &str) -> Vec<String> {
        return self.tokenize_to_vec(path);
    }


    fn tokenize_to_vec(&self, path: &str) -> Vec<String> {
        let mut tokens: Vec<String> = Vec::new();
        if path.is_empty() {
            return tokens;
        }

        path.split(self.path_separator.as_str())
            .for_each(|mut dir| {
                if self.trim_tokens {
                    dir = dir.trim()
                }
                if !dir.is_empty() {
                    tokens.push(dir.to_string())
                }
            });

        return tokens;
    }


    fn deactivate_pattern_cache(&mut self) {
        self.cache_patterns = Some(false);
        self.tokenized_pattern_cache.clear();
        self.string_matcher_cache.clear();
    }
}

impl Default for AntPathMatcher {
    fn default() -> Self {
        AntPathMatcher {
            path_separator: DEFAULT_PATH_SEPARATOR.to_string(),
            case_sensitive: false,
            trim_tokens: false,
            cache_patterns: None,
            tokenized_pattern_cache: HashMap::default(),
            string_matcher_cache: HashMap::default(),
        }
    }
}

impl PathMatcher for AntPathMatcher {
    fn is_pattern(&self, path: &str) -> bool {
        let mut uri_var = false;

        for i in 0..path.len() {
            let c = path.chars().nth(i).unwrap();
            if c == '*' || c == '?' {
                return true;
            }
            if c == '{' {
                uri_var = true;
                continue;
            }
            if c == '}' && uri_var {
                return true;
            }
        }
        return false;
    }

    fn match_(&mut self, pattern: &str, path: &str) -> bool {
        let mut uri_template_variables = None::<HashMap<String, String>>;
        self.do_match(pattern, path, true, &mut uri_template_variables)
    }

    fn match_start(&mut self, pattern: &str, path: &str) -> bool {
        let mut uri_template_variables = None::<HashMap<String, String>>;
        self.do_match(pattern, path, false, &mut uri_template_variables)
    }

    fn extract_path_within_pattern(&mut self, pattern: &str, path: &str) -> String {
        let pattern_parts = self.tokenize_to_vec(pattern);
        let path_parts = self.tokenize_to_vec(path);
        let mut builder = String::new();
        let mut path_started = false;
        for segment in 0..pattern_parts.len() {
            let pattern_part = &pattern_parts[segment];
            if pattern_part.contains('*') || pattern_part.contains('?') {
                for _ in 0..path_parts.len() {
                    if path_started || (segment == 0 && !pattern.starts_with(self.path_separator.as_str())) {
                        builder.push_str(self.path_separator.as_str());
                    }
                    builder.push_str(&path_parts[segment]);
                    path_started = true;
                }
            }
        }

        return builder;
    }

    fn extract_uri_template_variables(
        &mut self,
        pattern: &str,
        path: &str,
    ) -> HashMap<String, String> {
        let mut variables: Option<HashMap<String, String>> = Some(HashMap::new());
        let result = self.do_match(pattern, path, true, &mut variables);
        if result == false {
            panic!("Pattern \"{}\" is not a match for \"{}\"", pattern, path);
        }
        return variables.unwrap().clone();
    }
}

#[derive(Clone)]
pub struct AntPathStringMatcher {
    raw_pattern: String,
    exact_match: bool,

    case_sensitive: bool,
    regex: Option<Regex>,
    variable_names: Vec<String>,
}


impl AntPathStringMatcher {
    fn new(pattern: &str, case_sensitive: bool) -> Self {
        let mut pattern_builder = String::new();
        let mut variable_names: Vec<String> = Vec::new();
        let mut end: usize = 0;
        for matcher in Regex::new(GLOB_PATTERN).unwrap().find_iter(pattern) {
            pattern_builder.push_str(quote(pattern, end, matcher.start()).as_str());
            let match_ = matcher.as_str();

            if "?" == match_ {
                pattern_builder.push_str(".");
            } else if "*" == match_ {
                pattern_builder.push_str(".*");
            } else if match_.starts_with("{") && match_.ends_with("}") {
                let colon_idx = match_.find(':');

                match colon_idx {
                    None => {
                        pattern_builder.push_str(DEFAULT_VARIABLE_PATTERN);
                        variable_names.push(match_.trim_matches('{').trim_matches('}').to_string());
                    }
                    Some(idx) => {
                        let variable_pattern = match_.substring(idx + 1, match_.len() - 1);
                        pattern_builder.push_str("(");
                        pattern_builder.push_str(variable_pattern);
                        pattern_builder.push_str(")");
                        let variable_name = match_.substring(1, idx);
                        variable_names.push(variable_name.to_string());
                    }
                }
            }
            end = matcher.end();
        }
        return if end == 0 {
            AntPathStringMatcher {
                raw_pattern: pattern.to_string(),
                exact_match: true,
                case_sensitive,
                regex: None,
                variable_names,
            }
        } else {
            pattern_builder.push_str(quote(pattern, end, pattern.len()).as_str());

            let regex: Option<Regex>;
            if case_sensitive {
                regex = Option::Some(Regex::new(&pattern_builder).unwrap());
            } else {
                //convert to upper case
                regex = Option::Some(
                    RegexBuilder::new(&pattern_builder)
                        .case_insensitive(true)
                        .build()
                        .unwrap(),
                );
            }

            AntPathStringMatcher {
                raw_pattern: pattern_builder,
                exact_match: false,
                case_sensitive,
                regex,
                variable_names,
            }
        };
    }

    pub fn match_string(
        &self,
        str: &str,
         uri_template_variables: &mut Option<HashMap<String, String>>,
    ) -> bool {
        if self.exact_match {
            return if self.case_sensitive {
                self.raw_pattern == str
            } else {
                self.raw_pattern.to_uppercase() == str.to_uppercase()
            };
        } else if self.regex.is_some() {
            let regex = self.regex.as_ref().unwrap();

            if matches(regex, str) {
                if uri_template_variables.is_some() {
                    let captures = regex.captures(str).unwrap();
                    println!("regex: {:?}", regex);
                    println!("captures: {:?}", captures);
                    if self.variable_names.len() != captures.len() - 1 {
                        panic!("The number of capturing groups in the pattern segment {:?} does not match the number of URI template variables it defines, which can occur if capturing groups are used in a URI template regex. Use non-capturing groups instead.", regex);
                    }

                    for i in 1..=captures.len() - 1 {
                        let name = self.variable_names[i - 1].clone();
                        if name.starts_with("**") {
                            panic!("Capturing patterns ({:?}) are not supported by the AntPathMatcher. Use the PathPatternParser instead.", name);
                        }
                        let value = captures.get(i).unwrap().as_str().to_string();
                        uri_template_variables
                            .as_mut()
                            .unwrap()
                            .insert(name, value);
                    }
                }
                return true;
            }
        }
        return false;
    }
}

fn quote(s: &str, start: usize, end: usize) -> String {
    if start == end {
        return "".to_string();
    }
    return regex::escape(s.substring(start, end));
}

fn matches(regex: &Regex, s: &str) -> bool {
    match regex.find(s) {
        None => {
            return false;
        }
        Some(matcher) => matcher.end() == s.len(),
    }
}

