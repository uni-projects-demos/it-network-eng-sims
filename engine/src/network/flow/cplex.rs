use quick_xml::{Reader, events::Event};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub struct CplexMetrics {
    pub status: String,
    pub obj: f64,
    pub loads: Vec<f64>,
    pub nonzero_links: usize,
    pub max_cap: f64,
    pub highest_links: Vec<String>,
}

pub fn read_solution(xml: &str, transits: usize) -> Result<CplexMetrics, String> {
    let mut reader = Reader::from_str(xml);
    let mut status = None;
    let mut obj = None;
    let mut variables = BTreeMap::new();
    loop {
        match reader.read_event().map_err(|err| err.to_string())? {
            Event::Start(element) | Event::Empty(element) => {
                if ![b"header".as_slice(), b"variable".as_slice()]
                    .contains(&element.name().as_ref())
                {
                    continue;
                }
                let mut attributes = BTreeMap::new();
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|err| err.to_string())?;
                    attributes.insert(
                        String::from_utf8_lossy(attribute.key.as_ref()).into_owned(),
                        attribute
                            .decode_and_unescape_value(reader.decoder())
                            .map_err(|err| err.to_string())?
                            .into_owned(),
                    );
                }
                if element.name().as_ref() == b"header" {
                    status = attributes.get("solutionStatusString").cloned();
                    obj = attributes
                        .get("objectiveValue")
                        .and_then(|value| value.parse::<f64>().ok());
                } else if let (Some(name), Some(value)) =
                    (attributes.get("name"), attributes.get("value"))
                {
                    let value: f64 = value
                        .parse()
                        .map_err(|_| format!("Invalid CPLEX value for {name}"))?;
                    if !value.is_finite() || value < -1e-6 {
                        return Err(format!("Invalid CPLEX value for {name}"));
                    }
                    variables.insert(name.clone(), value);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let obj = obj
        .filter(|r| r.is_finite() && *r >= 0.0)
        .ok_or("CPLEX XML has no feasible objective")?;
    let status = status.ok_or("CPLEX XML has no solution status")?;
    let loads: Vec<f64> = (1..=transits)
        .map(|k| *variables.get(&format!("lK{k}")).unwrap_or(&0.0))
        .collect();
    if loads.iter().any(|&load| load > obj + 1e-5) {
        return Err("CPLEX loads exceed objective".into());
    }
    let caps: Vec<_> = variables
        .iter()
        .filter(|(name, _)| name.starts_with("cI") || name.starts_with("dK"))
        .collect();
    let max_cap = caps
        .iter()
        .map(|(_, value)| **value)
        .fold(0.0_f64, f64::max);
    Ok(CplexMetrics {
        status,
        obj,
        loads,
        nonzero_links: caps.iter().filter(|(_, value)| **value > 1e-7).count(),
        max_cap,
        highest_links: caps
            .iter()
            .filter(|(_, value)| (**value - max_cap).abs() < 1e-7)
            .map(|(name, _)| (*name).clone())
            .collect(),
    })
}
