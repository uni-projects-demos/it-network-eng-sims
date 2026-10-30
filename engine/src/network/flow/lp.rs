use super::model::{FlowConfig, PATHS};
use std::fmt::Write;

pub fn lp_filename(cfg: &FlowConfig) -> String {
    format!("X={}Y={}Z={}.lp", cfg.srcs, cfg.trans, cfg.dests)
}

fn write_sum(lp: &mut String, terms: impl Iterator<Item = String>) {
    lp.push_str("  ");
    for (idx, term) in terms.enumerate() {
        if idx > 0 {
            lp.push_str(" + ");
        }
        lp.push_str(&term);
    }
}

pub fn gen_lp(cfg: &FlowConfig) -> Result<String, String> {
    cfg.validate()?;
    let mut lp = String::from("MIN\n  r\nST\n");
    for i in 1..=cfg.srcs {
        for j in 1..=cfg.dests {
            write_sum(&mut lp, (1..=cfg.trans).map(|k| format!("xI{i}K{k}J{j}")));
            writeln!(lp, " = {}", i + j).unwrap();
        }
    }
    for i in 1..=cfg.srcs {
        for k in 1..=cfg.trans {
            write_sum(&mut lp, (1..=cfg.dests).map(|j| format!("xI{i}K{k}J{j}")));
            writeln!(lp, " - cI{i}K{k} = 0").unwrap();
        }
    }
    for k in 1..=cfg.trans {
        for j in 1..=cfg.dests {
            write_sum(&mut lp, (1..=cfg.srcs).map(|i| format!("xI{i}K{k}J{j}")));
            writeln!(lp, " - dK{k}J{j} = 0").unwrap();
        }
    }
    for i in 1..=cfg.srcs {
        for j in 1..=cfg.dests {
            write_sum(&mut lp, (1..=cfg.trans).map(|k| format!("uI{i}K{k}J{j}")));
            writeln!(lp, " = {PATHS}").unwrap();
        }
    }
    for k in 1..=cfg.trans {
        write_sum(
            &mut lp,
            (1..=cfg.srcs).flat_map(|i| (1..=cfg.dests).map(move |j| format!("xI{i}K{k}J{j}"))),
        );
        writeln!(lp, " - lK{k} = 0").unwrap();
    }
    for i in 1..=cfg.srcs {
        for j in 1..=cfg.dests {
            for k in 1..=cfg.trans {
                writeln!(lp, "  {PATHS} xI{i}K{k}J{j} - {} uI{i}K{k}J{j} = 0", i + j).unwrap();
            }
        }
    }
    for k in 1..=cfg.trans {
        write_sum(&mut lp, (1..=cfg.srcs).map(|i| format!("cI{i}K{k}")));
        lp.push_str(" - r <= 0\n");
    }
    lp.push_str("BOUND\n");
    for i in 1..=cfg.srcs {
        for k in 1..=cfg.trans {
            for j in 1..=cfg.dests {
                writeln!(lp, "  0 <= xI{i}K{k}J{j}").unwrap();
            }
        }
    }
    lp.push_str("  0 <= r\nBIN\n");
    for i in 1..=cfg.srcs {
        for j in 1..=cfg.dests {
            for k in 1..=cfg.trans {
                writeln!(lp, "  uI{i}K{k}J{j}").unwrap();
            }
        }
    }
    lp.push_str("END\n");

    let mut wrapped = String::with_capacity(lp.len());
    for line in lp.lines() {
        if line.len() <= 500 {
            wrapped.push_str(line);
        } else {
            wrapped.push_str("  ");
            let mut width = 2;
            let mut words = line.split_whitespace();
            while let Some(word) = words.next() {
                let token = if matches!(word, "=" | "<=" | ">=") {
                    format!("{word} {}", words.by_ref().collect::<Vec<_>>().join(" "))
                } else {
                    word.to_string()
                };
                if width > 2 && width + 1 + token.len() > 500 {
                    wrapped.push_str("\n  ");
                    width = 2;
                } else if width > 2 {
                    wrapped.push(' ');
                    width += 1;
                }
                wrapped.push_str(&token);
                width += token.len();
            }
        }
        wrapped.push('\n');
    }
    Ok(wrapped)
}
