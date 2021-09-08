use indoc::indoc;

use crate::{MdbookSnips, MdbookSnipsConfig};

#[test]
fn use_statements() {
    let r = &*super::IS_USE_STMT;

    assert!(r.is_match("use a;"));
    assert!(r.is_match("use "));
    assert!(r.is_match("pub use a;"));
    assert!(r.is_match("pub(crate) use a;"));
    assert!(r.is_match("pub(crate)use a;"));
    assert!(!r.is_match("pubuse a;"));
    assert!(r.is_match("pub(in crate) use a;"));
    assert!(r.is_match("pub(in crate::module) use a;"));
    assert!(r.is_match("pub(in ::crate) use a;"));
}

#[test]
fn doesnt_change_code_without_hidden_lines() {
    let mut content = String::from(indoc!(
        r#"
            # Hello
            ```
            fn main() {} // normal rust code
            ```
            "#
    ));

    let config = MdbookSnipsConfig::default();

    let preprocessor = MdbookSnips::new();

    assert_eq!(Ok(()), preprocessor.handle_content(&config, &mut content));

    assert_eq!(
        &content,
        indoc!(
            r#"
            # Hello
            ```
            fn main() {} // normal rust code
            ```
            "#
        )
    )
}

#[test]
fn adds_for_imports_when_configured() {
    let mut content = String::from(indoc!(
        r#"
            # Hello
            ```
            # pub(crate)use a;
            use b;
            # use c;
            use d;
            ```
            "#
    ));

    let config = MdbookSnipsConfig {
        for_imports: true,
        ..Default::default()
    };

    let preprocessor = MdbookSnips::new();

    assert_eq!(Ok(()), preprocessor.handle_content(&config, &mut content));

    assert_eq!(
        &content,
        indoc!(
            r#"
            # Hello
            ```
            // --snip--
            # pub(crate)use a;
            use b;
            // --snip--
            # use c;
            use d;
            ```
            "#
        )
    )
}

#[test]
fn does_not_add_for_imports_when_configured() {
    let mut content = String::from(indoc!(
        r#"
            # Hello
            ```
            # pub(crate)use a;
            use b;
            # use c;
            use d;
            ```
            "#
    ));

    let config = MdbookSnipsConfig {
        for_imports: false,
        ..Default::default()
    };

    let preprocessor = MdbookSnips::new();

    assert_eq!(Ok(()), preprocessor.handle_content(&config, &mut content));

    assert_eq!(
        &content,
        indoc!(
            r#"
            # Hello
            ```
            # pub(crate)use a;
            use b;
            # use c;
            use d;
            ```
            "#
        )
    )
}

#[test]
fn respects_indentation() {
    let mut content = String::from(indoc!(
        r#"
            # Hello
            ```
            # fn f() {}

            fn main() {
            #     let x = f();

                x
            }
            ```
            "#
    ));

    let config = MdbookSnipsConfig::default();

    let preprocessor = MdbookSnips::new();

    assert_eq!(Ok(()), preprocessor.handle_content(&config, &mut content));

    assert_eq!(
        &content,
        indoc!(
            r#"
            # Hello
            ```
            // --snip--
            # fn f() {}

            fn main() {
                // --snip--
            #     let x = f();

                x
            }
            ```
            "#
        )
    )
}

#[test]
fn ignores_whitespace_before_hash() {
    let mut content = String::from(indoc!(
        r#"
            # Hello
            ```
            # fn f() {}

            fn main() {
                # let x = f();

                x
            }
            ```
            "#
    ));

    let config = MdbookSnipsConfig::default();

    let preprocessor = MdbookSnips::new();

    assert_eq!(Ok(()), preprocessor.handle_content(&config, &mut content));

    assert_eq!(
        &content,
        indoc!(
            r#"
            # Hello
            ```
            // --snip--
            # fn f() {}

            fn main() {
                // --snip--
                # let x = f();

                x
            }
            ```
            "#
        )
    )
}