use crate::config::Config;
use crate::page::{IdSlugUid, Page, TitleAndUid, TitleSlugUid};
use crate::roam_edn::*;
use crate::syntax_highlight;
use anyhow::{anyhow, Result};
use fxhash::{FxHashMap, FxHashSet};
use itertools::Itertools;
use rayon::prelude::*;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize, Debug)]
struct TemplateArgs<'a> {
    title: &'a str,
    body: &'a str,
    tags: Vec<&'a str>,
    created_time: usize,
    edited_time: usize,
}

fn title_to_slug(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '/' || c == '-' || c == ':')
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphabetic() || c.is_digit(10))
                .flat_map(|c| c.to_lowercase())
                .collect::<String>()
        })
        .filter(|w| !w.is_empty())
        .join("_")
}

pub fn make_pages<'a, 'b>(
    graph: &'a Graph,
    handlebars: &handlebars::Handlebars,
    highlighter: &'b syntax_highlight::Highlighter,
    config: &Config,
) -> Result<FxHashMap<String, TitleAndUid>> {
    let mut all_filter_tags = vec![config.include.to_string()];
    all_filter_tags.extend_from_slice(&config.also);

    let tag_node_ids = all_filter_tags
        .iter()
        .map(|tag| {
            graph
                .titles
                .get(tag)
                .copied()
                .ok_or_else(|| anyhow!("Could not find page with filter name {}", tag))
        })
        .collect::<Result<Vec<usize>>>()?;

    let exclude_tag_ids = config
        .exclude
        .iter()
        .map(|tag| {
            graph
                .titles
                .get(tag)
                .copied()
                .ok_or_else(|| anyhow!("Could not find page with excluded filter name {}", tag))
        })
        .collect::<Result<Vec<usize>>>()?;

    let excluded_page_ids = graph
        .blocks_with_references(&exclude_tag_ids)
        .map(|block| block.page)
        .chain(exclude_tag_ids.iter().copied())
        .collect::<FxHashSet<usize>>();

    // let block_iter = if config.include_all {
    //   graph.pages()
    // } else {

    // }

    let main_tag_uid = graph
        .titles
        .get(&config.include)
        .and_then(|tag| graph.blocks.get(tag))
        .map(|b| b.uid.as_str());

    let tags_attr_uid = graph
        .titles
        .get(&config.tags_attr)
        .and_then(|tag| graph.blocks.get(tag))
        .map(|b| b.uid.as_str())
        .ok_or_else(|| anyhow!("Could not find tags attribute {}", config.tags_attr))?;

    let included_pages_by_title = graph
        .blocks
        .iter()
        .filter_map(|(_, block)| {
            if !config.include_all && !block.refs.iter().any(|r| tag_node_ids.contains(r)) {
                return None;
            }

            let page = graph.blocks.get(&block.page)?;
            page.title.as_ref()?;

            if excluded_page_ids.get(&page.id).is_some()
                || (page.log_id > 0 && !config.allow_daily_notes)
            {
                println!("Excluded: {}", page.title.as_ref().unwrap());
                return None;
            }

            let slug = match main_tag_uid.and_then(|uid| page.referenced_attrs.get(uid)) {
                // The page sets the filename manually.
                Some(AttrValue::Str(s)) => s.clone(),
                // Otherwise generate it from the title.
                _ => title_to_slug(page.title.as_ref().unwrap()),
            };

            Some((
                page.title.clone().unwrap(),
                IdSlugUid {
                    id: page.id,
                    slug,
                    uid: block.uid.clone(),
                },
            ))
        })
        .collect::<FxHashMap<_, _>>();

    let included_pages_by_id = included_pages_by_title
        .iter()
        .map(|(title, IdSlugUid { id, slug, uid })| {
            (
                *id,
                TitleSlugUid {
                    title: title.clone(),
                    slug: slug.clone(),
                    uid: uid.clone(),
                },
            )
        })
        .collect::<FxHashMap<_, _>>();

    let pages = included_pages_by_title
        .par_iter()
        .map(|(title, IdSlugUid { id, slug, uid })| {
            let mut output_path = config.output.join(slug);
            output_path.set_extension(&config.extension);

            let page = Page {
                id: *id,
                title: title.clone(),
                graph: &graph,
                omit_blocks_with_only_unexported_links: config
                    .omit_blocks_with_only_unexported_links,
                filter_tag: &config.include,
                included_pages_by_title: &included_pages_by_title,
                included_pages_by_id: &included_pages_by_id,
                highlighter,
            };

            let rendered = page.render()?;

            let block = graph.blocks.get(id).unwrap();

            let tags = block
                .referenced_attrs
                .get(tags_attr_uid)
                .and_then(|attr| match attr {
                    AttrValue::Str(s) => Some(s.split(",").map(|s| s.trim()).collect::<Vec<_>>()),
                    _ => None,
                })
                .unwrap_or_else(Vec::new);

            let template_data = TemplateArgs {
                title,
                body: &rendered,
                tags,
                created_time: block.create_time,
                edited_time: block.edit_time,
            };
            let full_page = handlebars.render("page", &template_data)?;

            let mut writer = std::fs::File::create(output_path)?;
            writer.write_all(full_page.as_bytes())?;
            writer.flush()?;

            println!("Wrote: \"{}\" to {}", title, slug);

            Ok((
                slug.clone(),
                TitleAndUid {
                    title: title.clone(),
                    uid: uid.clone(),
                },
            ))
        })
        .collect::<Result<FxHashMap<_, _>>>()?;

    let manifest_path = config.output.join("manifest.json");
    let mut manifest_writer = std::fs::File::create(manifest_path)?;
    serde_json::to_writer_pretty(&manifest_writer, &pages)?;
    manifest_writer.flush()?;
    drop(manifest_writer);

    Ok(pages)
}
