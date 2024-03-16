use std::collections::{HashMap, HashSet};

use clap::builder::Styles;

#[::linkme::distributed_slice]
pub static PARAMS: [(&str, &str)];

pub fn generate_params_help() -> String {
    let mut params: HashMap<String, HashSet<String>> = HashMap::default();
    for kv in PARAMS {
        params
            .entry(kv.0.to_string())
            .and_modify(|s| {
                s.insert(kv.1.to_string());
            })
            .or_insert(HashSet::from([kv.1.to_string()]));
    }
    let mut params: Vec<_> = params
        .iter()
        .map(|kv| {
            let mut descs = Vec::from_iter(kv.1.iter().map(|x| x.clone()));
            descs.sort();
            (kv.0.clone(), descs.join("\n\t"))
        })
        .collect();
    params.sort_by_key(|x| x.0.clone());

    let styles = Styles::default();
    let header = styles.get_header();
    let literal = styles.get_literal();
    format!(
        "{}Hyperparameters:{}\n",
        header.render(),
        header.render_reset()
    ) + &params
        .iter()
        .map(|kv| {
            format!(
                "  {}{}{}\n\t{}",
                literal.render(),
                kv.0,
                literal.render_reset(),
                kv.1
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}
