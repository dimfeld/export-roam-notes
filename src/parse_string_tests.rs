#[cfg(test)]
use crate::parse_string::{Expression::*, *};

#[test]
fn word() {
    let input = "word";
    assert_eq!(parse(input).unwrap(), vec![Expression::Text("word")])
}

#[test]
fn words() {
    let input = "two words";
    assert_eq!(parse(input).unwrap(), vec![Expression::Text("two words")])
}

#[test]
fn surrounding_whitespace() {
    let input = "  two words  ";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Text("  two words  ")]
    )
}

#[test]
fn block_ref() {
    let input = "((a ref))";
    assert_eq!(parse(input).unwrap(), vec![Expression::BlockRef("a ref")])
}

#[test]
fn link() {
    let input = "[[a title]]";
    assert_eq!(parse(input).unwrap(), vec![Expression::Link("a title")])
}

#[test]
fn hashtag_simple() {
    let input = "#tag";
    assert_eq!(parse(input).unwrap(), vec![Hashtag("tag", false)])
}

#[test]
fn hashtag_with_link() {
    let input = "#[[a tag]]";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Hashtag("a tag", false)]
    )
}

#[test]
fn hashtag_with_dot() {
    let input = "#.tag";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Hashtag("tag", true)]
    )
}

#[test]
fn other_brace() {
    let input = "{{ something-else }}";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::BraceDirective("something-else")]
    )
}

#[test]
fn table_brace() {
    let input = "{{ table }}";
    assert_eq!(parse(input).unwrap(), vec![Table])
}

#[test]
fn hashtag_brace() {
    // This isn't valid in Roam, so it doesn't parse out the hashtag.
    let input = "{{ #table}}";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::BraceDirective("#table")]
    )
}

#[test]
fn link_with_enclosed_bracket() {
    let input = "[[ab[cd]ef]]";
    assert_eq!(parse(input).unwrap(), vec![Expression::Link("ab[cd]ef")])
}

#[test]
fn table_link_brace() {
    let input = "{{[[table]]}}";
    assert_eq!(parse(input).unwrap(), vec![Table])
}

#[test]
fn other_link_brace() {
    let input = "{{[[something-else]]}}";
    assert_eq!(
        parse(input).unwrap(),
        vec![BraceDirective("something-else")]
    )
}

#[test]
fn multiword_with_links() {
    let input = "I want an [[astrolabe]] of my own";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("I want an "),
            Expression::Link("astrolabe"),
            Expression::Text(" of my own")
        ]
    )
}

#[test]
fn single_brace() {
    let input = "this is not [a brace ] but [[this is]]";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("this is not [a brace ] but "),
            Expression::Link("this is")
        ]
    )
}

#[test]
fn single_bracket() {
    let input = "this is not {a bracket } but [[this is a]]link";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("this is not {a bracket } but "),
            Expression::Link("this is a"),
            Expression::Text("link")
        ]
    )
}

#[test]
fn fake_bold() {
    let input = "this is *not* bold";
    assert_eq!(parse(input).unwrap(), vec![Text("this is *not* bold")]);
}

#[test]
fn image() {
    let input =
    "![](https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc)";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Image {
      alt: "",
      url: "https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc"
    }]
    )
}

#[test]
fn image_with_alt() {
    let input =
    "![some alt text](https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc)";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Image {
      alt: "some alt text",
      url: "https://firebasestorage.googleapis.com/v0/b/firescript-577a2.appspot.com/o/some-id?abc"
    }]
    )
}

#[test]
fn real_world_1() {
    let input = r##"An initially \"honest\" signal becomes dishonest."##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Text(
            r##"An initially \"honest\" signal becomes dishonest."##
        )]
    )
}

#[test]
fn plaintext_link() {
    let input = r##"Source: https://a.website.com/is-post"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text(r##"Source: "##),
            RawHyperlink("https://a.website.com/is-post")
        ]
    )
}

#[test]
fn plaintext_link_entire_string() {
    let input = "https://www.example.com/def/ghi?abc=def#an-anchor";
    assert_eq!(parse(input).unwrap(), vec![Expression::RawHyperlink(input)]);
}

#[test]
fn plaintext_link_omits_trailing_character() {
    let input = "at https://www.example.com/def.";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Text("at "),
            RawHyperlink("https://www.example.com/def"),
            Text(".")
        ]
    );
}

#[test]
fn plaintext_link_omits_trailing_character2() {
    let input = "at https://www.example.com/def/ghi?abc=def#an-anchor.";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Text("at "),
            RawHyperlink("https://www.example.com/def/ghi?abc=def#an-anchor"),
            Text(".")
        ]
    );
}

#[test]
fn markdown_link() {
    let input =
        r##"For actually communicating, [spiped](https://www.tarsnap.com/spiped.html) is nice"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("For actually communicating, "),
            Expression::MarkdownLink {
                title: "spiped",
                url: "https://www.tarsnap.com/spiped.html"
            },
            Expression::Text(" is nice")
        ]
    )
}

#[test]
fn attribute_simple() {
    let input = "Source:: some blog";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Attribute {
            name: "Source",
            value: vec![Expression::Text("some blog")]
        }]
    )
}

#[test]
fn attribute_nospace() {
    let input = "Source::some blog";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Attribute {
            name: "Source",
            value: vec![Expression::Text("some blog")]
        }]
    )
}

#[test]
fn attribute_complex() {
    let input = " My Score:: too [[high]] to count";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Attribute {
            name: " My Score",
            value: vec![
                Expression::Text("too "),
                Expression::Link("high"),
                Expression::Text(" to count")
            ]
        }]
    )
}

#[test]
fn attribute_extra_colons() {
    let input = " My Score::: too :: high :: to count";
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Attribute {
            name: " My Score",
            value: vec![Expression::Text(": too :: high :: to count"),]
        }]
    )
}

#[test]
fn attribute_backticks_1() {
    // Do not parse it as an attribute if the :: is inside backticks
    let input = " My Score ` :: too [[high]] to count`";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text(" My Score "),
            Expression::SingleBacktick(" :: too [[high]] to count")
        ]
    )
}

#[test]
fn attribute_backticks_2() {
    // This feels weird but it matches Roam's behavior.
    // Understandable since it's difficult to parse otherwise
    let input = "My `Score`:: too [[high]] to count";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("My "),
            Expression::SingleBacktick("Score"),
            Expression::Text(":: too "),
            Expression::Link("high"),
            Expression::Text(" to count")
        ]
    )
}

#[test]
fn exclamation_point() {
    let input = "This is exciting!";
    assert_eq!(parse(input).unwrap(), vec![Text("This is exciting!")]);
}

#[test]
fn real_world_2() {
    let input = "Added support for switchable transition styles to [[svelte-zoomable]]";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("Added support for switchable transition styles to "),
            Expression::Link("svelte-zoomable")
        ]
    )
}

#[test]
fn real_world_3() {
    let input = "Include `hostnames;` inside the block to let it do wildcard matches on hostnames.";
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("Include "),
            Expression::SingleBacktick("hostnames;"),
            Expression::Text(" inside the block to let it do wildcard matches on hostnames.")
        ]
    )
}

#[test]
fn real_world_4() {
    let input = r##"**Algorithm - Difference Engine** #roam/templates"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Bold(vec![Text("Algorithm - Difference Engine")]),
            Text(" "),
            Hashtag("roam/templates", false),
        ]
    )
}

#[test]
fn real_world_5() {
    let input = r##"{{[[TODO]]}} [[Projects/Rewrite everything]]"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::BraceDirective("TODO"),
            Expression::Text(" "),
            Expression::Link("Projects/Rewrite everything"),
        ]
    )
}

#[test]
fn real_world_6() {
    let input = r##"{{[[TODO]]}}[[Projects/Rewrite everything]]"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::BraceDirective("TODO"),
            Expression::Link("Projects/Rewrite everything"),
        ]
    )
}

#[test]
fn real_world_7() {
    let input =
        r##"([Location 1062](https://readwise.io/to_kindle?action=open&asin=2232&location=1062))"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::Text("("),
            Expression::MarkdownLink {
                title: "Location 1062",
                url: "https://readwise.io/to_kindle?action=open&asin=2232&location=1062"
            },
            Expression::Text(")"),
        ]
    )
}

#[test]
fn real_world_8() {
    let input = r##"--- **John 13:18-30 - Judas and Jesus** ---"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Text("--- "),
            Bold(vec![Text("John 13:18-30 - Judas and Jesus")]),
            Text(" ---")
        ]
    )
}

#[test]
fn triple_backtick_1() {
    let input = r##"```javascript\nmap $regex_domain $domain {\n  app defaultskin;\n  tm defaultskin;\n  www defaultskin;\n  '' defaultskin;\n  dev defaultskin;\n  default $regex_domain;\n}```"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::TripleBacktick(
            r##"javascript\nmap $regex_domain $domain {\n  app defaultskin;\n  tm defaultskin;\n  www defaultskin;\n  '' defaultskin;\n  dev defaultskin;\n  default $regex_domain;\n}"##
        )]
    )
}

#[test]
fn triple_backtick_2() {
    let input = r##"```css\nbackground: #203;\ncolor: #ffc;\ntext-shadow: 0 0 .1em, 0 0 .3em;```"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::TripleBacktick(
            r##"css\nbackground: #203;\ncolor: #ffc;\ntext-shadow: 0 0 .1em, 0 0 .3em;"##
        )]
    )
}

#[test]
fn todo() {
    let input = r##"{{[[TODO]]}} Get things done"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![
            Expression::BraceDirective("TODO"),
            Expression::Text(" Get things done")
        ]
    )
}

#[test]
fn unicode() {
    let input = r##"client’s merkle tree"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Text("client’s merkle tree")]
    )
}

#[test]
fn blockquote_simple() {
    let input = r##"> Some text"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::BlockQuote(vec![Expression::Text("Some text")])]
    );
}

#[test]
fn blockquote_with_nested_styles() {
    let input = r##"> [[Some]] **text**"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::BlockQuote(vec![
            Expression::Link("Some"),
            Expression::Text(" "),
            Expression::Bold(vec![Expression::Text("text")])
        ])]
    );
}

#[test]
fn blockquote_fake_1() {
    let input = r##" > Some text"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Text(" > Some text")]
    );
}

#[test]
fn blockquote_fake_2() {
    let input = r##"Some text
> and another"##;
    assert_eq!(
        parse(input).unwrap(),
        vec![Expression::Text("Some text\n> and another")]
    );
}
