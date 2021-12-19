use crate::path_matcher::{AntPathMatcher, PathMatcher};


#[test]
fn test_1() {
    let mut ant_path_matcher = AntPathMatcher::default();

    // let result = ant_path_matcher.match_("/", "/");
    // assert_eq!(result, true);
    //
    //
    // let result = ant_path_matcher.match_("/", "/a");
    // assert_eq!(result, false);
    //
    // let result = ant_path_matcher.match_("/a", "/a");
    //
    // assert!(result, true);
    // let result = ant_path_matcher.match_("/a/*", "/a/b");
    //
    // assert!(result, true);

    let result = ant_path_matcher.match_("/a/*", "/a/b/c");

    assert_eq!(result, false);
}