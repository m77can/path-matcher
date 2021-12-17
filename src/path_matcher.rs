// // pub mod path-matcher
//
// use regex::{Regex, Match};
// use std::collections::HashMap;
// use validator::HasLen;
// use std::string::ToString;
// use chashmap::ReadGuard;
//
//
// //?	匹配任何单字符
// //*	匹配0或者任意数量的字符
// //**	匹配0或者更多的目录
//
// static GLOB_PATTERN: &'static str = r"\?|\*|\{((?:\{[^/]+?\}|[^/{}]|\\[{}])+?)\}";
//
// static DEFAULT_VARIABLE_PATTERN: &'static str = "(.*)";
//
// static DEFAULT_PATH_SEPARATOR: &'static str = "/";
//
// static CACHE_TURNOFF_THRESHOLD: usize = 65536;
//
// static VARIABLE_PATTERN: &'static str = "\\{[^/]+?\\}";
//
// static WILDCARD_CHARS: [char; 3] = ['*', '?', '{'];
//
// pub trait PathMatcher {
//     fn is_pattern(&self, path: &str) -> bool;
//
//     fn r#match(&mut self, pattern: &str, path: &str) -> bool;
//
//     fn match_start(&self, pattern: String, path: String) -> bool;
//     fn extract_path_within_pattern(&mut self, pattern: &str, path: &str) -> String;
//
//     fn extract_uri_template_variables(&mut self, pattern: &str, path: &str) -> HashMap<String, String>;
//
//     fn combine(&self, pattern1: String, pattern2: String) -> String;
// }
//
// pub struct AntPathMatcher {
//     path_separator: String,
//     path_separator_pattern_cache: PathSeparatorPatternCache,
//     case_sensitive: bool,
//     trim_tokens: bool,
//     cache_patterns: bool,
//     tokenized_pattern_cache: chashmap::CHashMap<String, Vec<String>>,
//     string_matcher_cache: chashmap::CHashMap<String, AntPathStringMatcher>,
// }
//
// impl AntPathMatcher {
//     pub fn new(
//         path_separator: &str,
//         case_sensitive: bool,
//         trim_tokens: bool,
//         cache_patterns: bool,
//     ) -> Self {
//         AntPathMatcher {
//             path_separator: path_separator.to_string(),
//             path_separator_pattern_cache: PathSeparatorPatternCache::new(path_separator),
//             case_sensitive,
//             trim_tokens,
//             cache_patterns,
//             tokenized_pattern_cache: chashmap::CHashMap::new(),
//             string_matcher_cache: chashmap::CHashMap::new(),
//         }
//     }
//
//     fn do_match(&mut self, pattern: &String, path: &String, full_path: bool, uri_template_variables: &mut Option<HashMap<String, String>>) -> bool {
//         if path.starts_with(self.path_separator.as_str()) != pattern.starts_with(self.path_separator.as_str()) {
//             return false;
//         }
//
//         let patt_dirs = self.tokenize_pattern(pattern);
//
//         if full_path && self.case_sensitive && !self.is_potential_match(path, &patt_dirs) {
//             return false;
//         }
//
//         let path_dirs = self.tokenize_path(path);
//         let mut patt_idx_start: usize = 0;
//         let mut patt_idx_end = patt_dirs.len() - 1;
//         let mut path_idx_start: usize = 0;
//         let mut path_idx_end = path_dirs.len() - 1;
//
//         while path_idx_start <= patt_idx_end && path_idx_start <= path_idx_end {
//             let patt_dir = patt_dirs.get(patt_idx_start).unwrap();
//             if "**" == patt_dir {
//                 break;
//             }
//             if !self.match_string(patt_dir, path_dirs.get(path_idx_start).unwrap(), uri_template_variables) {
//                 return false;
//             }
//             patt_idx_start = patt_idx_start + 1;
//             path_idx_start = path_idx_start + 1;
//         }
//
//         if path_idx_start > path_idx_end {
//             if patt_idx_start > patt_idx_end {
//                 return pattern.ends_with(self.path_separator.as_str()) == path.ends_with(self.path_separator.as_str());
//             }
//             if !full_path {
//                 return true;
//             }
//             if patt_idx_start == patt_idx_end && patt_dirs.get(patt_idx_start).unwrap() == "*" && path.ends_with(self.path_separator.as_str()) {
//                 return true;
//             }
//             let mut i = patt_idx_start;
//
//             while i <= patt_idx_end {
//                 if patt_dirs.get(i).unwrap() != "**" {
//                     return false;
//                 }
//                 i = i + 1;
//             }
//             return true;
//         } else if patt_idx_start > patt_idx_end {
//             return false;
//         } else if !full_path && "**" == patt_dirs.get(patt_idx_start).unwrap() {
//             return true;
//         }
//
//         // up to last '**'
//         while patt_idx_start <= patt_idx_end && path_idx_start <= path_idx_end {
//             let patt_dir = patt_dirs.get(patt_idx_end).unwrap();
//
//             if patt_dir == "**" {
//                 break;
//             }
//
//             if !self.match_string(patt_dir, patt_dirs.get(path_idx_end).unwrap(), uri_template_variables) {
//                 return false;
//             }
//             path_idx_end = path_idx_end - 1;
//             patt_idx_end = patt_idx_end - 1;
//         }
//
//         if path_idx_start > path_idx_end {
//             let mut i = patt_idx_start;
//             while i <= patt_idx_end {
//                 if "**" != patt_dirs.get(i).unwrap() {
//                     return false;
//                 }
//                 i = i + 1;
//             }
//             return true;
//         }
//
//         while patt_idx_start != patt_idx_end && path_idx_start <= path_idx_end {
//             let mut pat_idx_tmp: i64 = -1;
//             let mut i = path_idx_start + 1;
//
//             while i <= patt_idx_end {
//                 if patt_dirs.get(i).unwrap() == "**" {
//                     pat_idx_tmp = i as i64;
//                     break;
//                 }
//                 i = i + 1;
//             }
//             if pat_idx_tmp == (patt_idx_start + 1) as i64 {
//                 // '**/**' situation, so skip one
//
//                 patt_idx_start = patt_idx_start + 1;
//                 continue;
//             }
//             // Find the pattern between padIdxStart & padIdxTmp in str between
//             // strIdxStart & strIdxEnd
//             let pat_length = pat_idx_tmp as usize - patt_idx_start - 1;
//             let str_length = path_idx_end - path_idx_start + 1;
//             let mut found_idx: i64 = -1;
//             'outer: for i in 0..(str_length - pat_length + 1) {
//                 for j in 0..pat_length {
//                     let sub_pat = patt_dirs.get(patt_idx_start + j + 1).unwrap();
//                     let sub_str = patt_dirs.get(patt_idx_start + i + j).unwrap();
//                     if !self.match_string(sub_pat, sub_str, uri_template_variables) {
//                         continue 'outer;
//                     }
//                 }
//                 found_idx = (path_idx_start + i) as i64;
//                 break;
//             }
//             if found_idx == -1 {
//                 return false;
//             }
//             patt_idx_start = pat_idx_tmp as usize;
//             path_idx_start = found_idx as usize + pat_length;
//         }
//
//         for i in patt_idx_start..(patt_idx_end + 1) {
//             println!("Result: {:?}", patt_dirs.get(i));
//             if patt_dirs.get(i).unwrap() != "**" {
//                 return false;
//             }
//         }
//
//         return true;
//     }
//
//     fn match_string(&mut self, pattern: &String, string: &String, uri_template_variables: &mut Option<HashMap<String, String>>) -> bool {
//         self.get_string_matcher(pattern).match_strings(string, uri_template_variables)
//     }
//
//
//     fn get_string_matcher(&mut self, pattern: &String) -> AntPathStringMatcher {
//         let cache_patterns = self.cache_patterns;
//         if cache_patterns {
//             return match self.string_matcher_cache.get(pattern) {
//                 Some(matcher) => {
//                     matcher.to_owned()
//                 }
//                 _ => {
//                     let matcher = AntPathStringMatcher::new(pattern, &self.case_sensitive);
//
//                     if self.string_matcher_cache.len() >= CACHE_TURNOFF_THRESHOLD {
//                         self.cache_patterns = false;
//                         self.tokenized_pattern_cache.clear();
//                         self.string_matcher_cache.clear();
//                         return matcher;
//                     }
//                     if cache_patterns {
//                         self.string_matcher_cache.insert(pattern.clone(), matcher.clone());
//                     }
//                     matcher
//                 }
//             };
//         }
//
//         AntPathStringMatcher::new(pattern, &self.case_sensitive)
//     }
//
//
//     fn tokenize_pattern(&mut self, pattern: &String) -> Vec<String> {
//         let mut tokenized: Vec<String> = Vec::new();
//         let cache_patterns = self.cache_patterns;
//
//         if cache_patterns {
//             println!("{:?}", self.tokenized_pattern_cache.get(pattern));
//
//             match self.tokenized_pattern_cache.get(pattern) {
//                 Some(entry) => {
//                     tokenized = entry.to_vec();
//                 }
//                 _ => {
//                     println!("nothing");
//                 }
//             }
//
//             match self.tokenized_pattern_cache.get(pattern) {
//                 None => {}
//                 Some(t) => { tokenized = t.to_vec() }
//             };
//         }
//         if tokenized.is_empty() {
//             tokenized = self.tokenize_path(pattern);
//             if self.tokenized_pattern_cache.len() >= CACHE_TURNOFF_THRESHOLD {
//                 self.cache_patterns = false;
//                 self.tokenized_pattern_cache.clear();
//                 self.string_matcher_cache.clear();
//                 return tokenized;
//             }
//             if cache_patterns {
//                 self.tokenized_pattern_cache.insert(pattern.clone(), tokenized.clone());
//             }
//         }
//         return tokenized;
//     }
//
//     fn tokenize_path(&self, path: &String) -> Vec<String> {
//         let delims: Vec<_> = self.path_separator.chars().collect();
//         let tokens: Vec<String> = path.split(&delims[..]).filter(|k| { !k.is_empty() }).map(|k| k.trim().to_string()).collect();
//
//         return tokens;
//     }
//
//     fn is_potential_match(&self, path: &String, patt_dirs: &Vec<String>) -> bool {
//         if self.trim_tokens {
//             let mut pos = 0;
//
//             for patt_dir in patt_dirs {
//                 let mut skipped = self.skip_separator(path, pos, &self.path_separator);
//                 pos = skipped + pos;
//                 skipped = self.skip_segment(path, pos, patt_dir);
//                 if skipped < patt_dir.len() {
//                     let patt_dir_chars: Vec<char> = patt_dir.chars().collect();
//                     return skipped > 0 || patt_dir.len() > 0 && self.is_wildcard_char(patt_dir_chars.get(0).unwrap());
//                 }
//                 pos = skipped + pos;
//             }
//         }
//         return true;
//     }
//
//     fn skip_separator(&self, path: &String, pos: usize, separator: &String) -> usize {
//         let mut skipped: usize = 0;
//
//         while starts_with(path, separator, (pos + skipped) as usize) {
//             skipped = separator.length() as usize + skipped;
//         }
//         return skipped;
//     }
//
//     fn skip_segment(&self, path: &String, pos: usize, prefix: &String) -> usize {
//         let prefix_list: Vec<char> = prefix.chars().collect();
//
//         let path_list: Vec<char> = path.chars().collect();
//
//         let mut skipped: usize = 0;
//         let mut i = 0;
//         while i < prefix.len() {
//             let c = prefix_list.get(i).unwrap();
//             if self.is_wildcard_char(&c) {
//                 return skipped;
//             }
//             let curr_pos = pos + skipped;
//             if curr_pos > path.length() as usize {
//                 return 0;
//             }
//             if c == path_list.get(curr_pos).unwrap() {
//                 skipped = skipped + 1;
//             }
//             i = i + 1;
//         }
//         return skipped;
//     }
//
//     fn is_wildcard_char(&self, c: &char) -> bool {
//         for candidate in WILDCARD_CHARS.iter() {
//             if c == candidate {
//                 return true;
//             }
//         }
//         return false;
//     }
// }
//
// impl PathMatcher for AntPathMatcher {
//     fn is_pattern(&self, path: &str) -> bool {
//         let mut uri_var = false;
//
//         for c in path.chars() {
//             println!("{:?}", c);
//             if c == '*' || c == '?' {
//                 return true;
//             }
//             if c == '{' {
//                 uri_var = true;
//                 continue;
//             }
//             if c == '}' && uri_var {
//                 return true;
//             }
//         }
//         false
//     }
//     fn r#match(&mut self, pattern: &str, path: &str) -> bool {
//         self.do_match(&pattern.to_string(), &path.to_string(), true, &mut None)
//     }
//     fn match_start(&self, _: std::string::String, _: std::string::String) -> bool {
//         unimplemented!()
//     }
//     fn extract_path_within_pattern(
//         &mut self, pattern: &str, path: &str,
//     ) -> std::string::String {
//         unimplemented!()
//     }
//
//     //    Map<String, String> variables = new LinkedHashMap<>();
// //    boolean result = doMatch(pattern, path, true, variables);
// //    if (!result) {
// //    throw new IllegalStateException("Pattern \"" + pattern + "\" is not a match for \"" + path + "\"");
// //    }
// //    return variables;
//     fn extract_uri_template_variables(
//         &mut self,
//         pattern: &str,
//         path: &str,
//     ) -> HashMap<String, String> {
//         let mut variables: Option<HashMap<String, String>> = Some(HashMap::new());
//         let result = self.do_match(&pattern.to_string(), &path.to_string(), true, &mut variables);
//
//         if !result {
//             panic!("ERROR for extract variables");
//         }
//         variables.unwrap()
//     }
//     fn combine(&self, _: std::string::String, _: std::string::String) -> std::string::String {
//         unimplemented!()
//     }
// }
//
//
// struct PatternInfo {
//     pattern: String,
//     uri_vars: i32,
//     single_wildcards: i32,
//     double_wildcards: i32,
//     catch_all_pattern: bool,
//     prefix_pattern: bool,
//     length: usize,
// }
//
// impl PatternInfo {
//     pub fn new(pattern: &mut String) -> Self {
//         let pattern: String = pattern.clone();
//         let uri_vars = 0;
//
//         let pattern_info = PatternInfo {
//             pattern,
//             uri_vars: 0,
//             single_wildcards: 0,
//             double_wildcards: 0,
//             catch_all_pattern: false,
//             prefix_pattern: false,
//             length: 0,
//         };
//         return pattern_info;
//     }
//
//     pub fn get_total_count(&self) -> i32 {
//         self.uri_vars + self.single_wildcards + self.double_wildcards * 2
//     }
//
//     // Returns the length of the given pattern, where template variables are considered to be 1 long.
//     pub fn get_length(&self) -> usize {
//         let after = Regex::new(VARIABLE_PATTERN)
//             .unwrap()
//             .replace_all(&self.pattern, "#");
//
//         after.chars().count()
//     }
// }
//
// #[derive(Clone)]
// struct AntPathStringMatcher {
//     pattern: Regex,
//     variable_names: Vec<String>,
// }
//
// impl AntPathStringMatcher {
//     fn new(pattern: &String, case_sensitive: &bool) -> Self {
//         let mut variable_names: Vec<String> = Vec::new();
//         let mut pattern_builder = String::new();
//         let re = Regex::new(GLOB_PATTERN).unwrap();
//
//         let mut end = 0;
//         for find in re.find_iter(pattern) {
//             println!("{:?}", find);
//             let matcher = find.as_str();
//             pattern_builder.push_str(&pattern[end..find.start()]);
//
//             if "?" == matcher {
//                 pattern_builder.push('.');
//             } else if "*" == matcher {
//                 pattern_builder.push_str(".*");
//             } else if matcher.starts_with("{") && matcher.ends_with("}") {
//                 match matcher.find(":") {
//                     Some(colon_idx) => {
//                         let variable_pattern = &matcher[colon_idx..matcher.len() - 1];
//                         pattern_builder.push('(');
//                         pattern_builder.push_str(variable_pattern);
//                         pattern_builder.push(')');
//                         let variable_name = &matcher[1..colon_idx];
//                         variable_names.push(variable_name.to_string());
//                     }
//
//                     _ => {
//                         pattern_builder.push_str(DEFAULT_VARIABLE_PATTERN);
//                         variable_names.push(find.as_str().to_string())
//                     }
//                 }
//             }
//             end = find.end();
//         }
//         pattern_builder.push_str(&pattern[end..pattern.len()]);
//
//         let ant_path_string_matcher = AntPathStringMatcher {
//             pattern: Regex::new(&pattern_builder).expect("Pattern is invalid"),
//             variable_names,
//         };
//         return ant_path_string_matcher;
//     }
//
//     fn match_strings(&self, str: &String, uri_template_variables: &mut Option<HashMap<String, String>>) -> bool {
//         println!("ant path string matcher match strings, string: {}, pattern: {:?}", str, self.pattern);
//         println!("str: {:?}, url_template_variable: {:?}", str, uri_template_variables);
//
//         if self.pattern.find_iter(str).filter(|matcher| str == matcher.as_str()).count() > 0 {
//             if !uri_template_variables.is_some() {
//                 if self.variable_names.len() != self.pattern.captures(str).unwrap().len() {
//                     panic!("Fuck")
//                 }
//                 for i in 1..(self.pattern.find_iter(str).count() + 1) {
//                     let name = self.variable_names.get(i - 1).unwrap();
//
//                     let values: Vec<Match> = self.pattern.find_iter(str).collect();
//                     let value = values.get(i).unwrap().as_str();
//
//                     uri_template_variables.take().unwrap().insert(name.clone(), value.to_string());
//                 }
//             }
//             return true;
//         } else {
//             return false;
//         }
//     }
// }
//
//
// struct PathSeparatorPatternCache {
//     ends_on_wildcard: String,
//     ends_on_double_wildcard: String,
// }
//
// impl PathSeparatorPatternCache {
//     fn new(path_separator: &str) -> Self {
//         let mut ends_on_wildcard = path_separator.to_string();
//         let mut ends_on_double_wildcard = path_separator.to_string();
//         ends_on_wildcard.push_str("*");
//         ends_on_double_wildcard.push_str("**");
//
//         PathSeparatorPatternCache {
//             ends_on_wildcard,
//             ends_on_double_wildcard,
//         }
//     }
// }
//
// //TODO
// fn quote(s: &String, start: usize, end: usize) -> String {
//     if start == end {
//         return "".to_string();
//     }
//     let mut sb = String::new();
//
//     let s = &s[start..end];
//
//     match s.find("\\t") {
//         Some(slash_E_index) => {
//             sb.push_str("\\t");
//             let slash_E_index = 0;
//             let mut current = 0;
//
//             let mut current_s = &s[0..s.len()];
//             while current_s.find("\\t").is_some() {
//                 let slash_E_index = current_s.find("\\t").unwrap();
//                 sb.push_str(&current_s[current..slash_E_index]);
//                 current = slash_E_index + 2;
//                 sb.push_str("\\t\\\\t\\t");
//                 current_s = &current_s[current..s.len()];
//             }
//
//             sb.push_str(&current_s[current..s.len()]);
//             sb.push_str("\\t");
//             return sb;
//         }
//         _ => {
//             let mut result = String::from("\\t");
//             result.push_str(s);
//             result.push_str("\\t");
//             result
//         }
//     }
// }
//
//
// #[test]
// fn test_tokenizer() {
//     let re = regex::Regex::new(r"a|,").unwrap();
//     for part in re.split("ab,aa,ba,cccca,ddd,eee,     aaa, a,") {
//         if !part.is_empty() {
//             println!("{}", part);
//         }
//     }
// }
//
//
// #[test]
// fn test_split() {
//     let input = "ab,aa,ba,cccca,ddd,eee,     aaa, a,";
//     let delim_string = "a,";
//
//     let delims: Vec<_> = delim_string.chars().collect();
//     let tokens: Vec<_> = input.split(&delims[..]).filter(|k| !k.is_empty()).collect();
//     for part in input.split(&delims[..]) {
//         if !part.is_empty() {
//             println!("{}", part);
//         }
//     }
// }
//
// fn starts_with(value: &String, prefix: &String, to_offset: usize) -> bool {
//     let ta: Vec<char> = value.chars().collect();
//     let mut to = to_offset;
//     let pa: Vec<char> = prefix.chars().collect();
//
//     let mut po: usize = 0;
//
//     let mut pc = pa.len();
//
//     if to_offset < 0 || to_offset > ta.len() - pc {
//         return false;
//     }
//
//     while pc > 0 {
//         if ta.get(to) != pa.get(po) {
//             return false;
//         }
//         to = to + 1;
//         po = po + 1;
//         pc = pc - 1;
//     }
//     return true;
// }
//
// #[test]
// fn test_string_matcher() {
//     let re = Regex::new(GLOB_PATTERN).unwrap();
//
//     for find in re.find_iter("{api}") {
//         println!("{:?}", find);
//     }
// }
//
// #[test]
// fn test_regex_find() {
//     let re = Regex::new(GLOB_PATTERN).unwrap();
//
//     match re.find("{api}") {
//         Some(t) => {
//             for i in t.range() {
//                 println!("{:?}", i)
//             }
//             println!("{:?}", t);
//         }
//
//         None => println!("None")
//     }
// }
//
// #[test]
// fn test_regex_wildcard() {
//     let re = Regex::new(r"a.*b.*c").unwrap();
//     if re.is_match("abcd") {
//         println!("matched");
//     }
// }
//
// #[test]
// fn test_regex_match() {
//     let re = Regex::new(r"a.*b.*c").unwrap();
//     if re.find_iter("abcd").filter(|mat: &Match| mat.as_str() == "abcd").count() > 0 {
//         println!("match");
//     } else {
//         println!("not match")
//     }
// }
//
// #[test]
// fn test_is_pattern() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {}", matcher.is_pattern("a,b"));
// }
//
// macro_rules! matches {
//         ($name:ident, $pat:expr, $path:expr) => {
//             #[test]
//             fn $name() {
//                 let mut matcher = AntPathMatcher::new(&mut "/".to_string(), false, true, true);
//                 let pattern = String::from($pat);
//                 let path = String::from($path);
//                 assert!(matcher.r#match(&pattern,&path));
//                 println!("pattern: {}, path: {}. matched",pattern,path);
//
//             }
//         };
//     }
//
// #[test]
// fn test_match_alt_1() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {}", matcher.r#match("a,b", "a,b"));
// }
//
// #[test]
// fn test_match_alt_2() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {}", matcher.r#match(",", ","));
// }
//
// #[test]
// fn test_match_alt_3() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {}", matcher.r#match("{a,b}", "a"));
// }
//
// #[test]
// fn test_match_alt_4() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {}", matcher.r#match("**/src/**", "abc/src/bar"));
// }
//
// #[test]
// fn test_match_alt_5() {
//     let mut matcher = AntPathMatcher::new("/", false, true, true);
//     println!("Result is: {:?}", matcher.extract_uri_template_variables("com/{filename:\\w+}.jsp", "com/test.jsp"));
// }
//
// matches!(match_1,"com/t?st.jsp","com/test.jsp");
//
// matches!(match_2,"com/t?st.jsp","com/tast.jsp");
//
// matches!(match_3,"com/t?st.jsp","com/txst.jsp");
//
// matches!(match_4,"com/*.jsp","com/test.jsp");
//
// matches!(match_5,"com/**/test.jsp","com/ab/test.jsp");
//
// matches!(match_6,"com/{filename:\\w+}.jsp","com/test.jsp");
//
//
// matches!(match_7,"/hello/**","/hello/me/it");
// //
// matches!(match_8,"/api/**","/api/users/id");