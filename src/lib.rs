use anyhow::{anyhow, bail, Context as _, Result};
use regex::Regex;
use std::env::args;

pub enum Significance {
    Five,
    One,
}

enum Probability {
    Raw(f64),
    Percent(f64),
    OneOutOf(f64),
    Ratio(f64, f64),
}

fn solve(p: Probability, s: Significance) -> f64 {
    let p = p.val();
    let s = s.val();
    // n such that:
    // (1 - p)^n < s
    // n * ln(1 - p) < ln(s)
    // n > ln(s) / ln(1 - p)
    let exact = s.ln() / (1. - p).ln();
    exact.ceil()
}

pub fn run(s: Significance) -> Result<()> {
    println!("{}", solve(parse_arg()?, s));
    Ok(())
}

impl Significance {
    fn val(&self) -> f64 {
        use Significance::*;
        match &self {
            Five => 0.05,
            One => 0.01,
        }
    }
}

impl Probability {
    fn val(&self) -> f64 {
        use Probability::*;
        match &self {
            Raw(v) => *v,
            Percent(v) => *v / 100.,
            OneOutOf(v) => 1. / *v,
            Ratio(n, d) => *n / *d,
        }
    }
}

fn parse_arg() -> Result<Probability> {
    use Probability::*;
    let args = args().skip(1).collect::<Vec<String>>();
    let mut succ_num = false;
    for a in &args {
        if a.parse::<f64>().is_ok() {
            if succ_num {
                bail!("successive numerical arguments");
            }
            succ_num = true;
        } else {
            succ_num = false;
        }
    }
    let arg = args.join("").trim().to_owned();
    let re_raw = Regex::new(r"^0?\.[0-9]+$").context("[I] re_raw build re")?;
    if re_raw.is_match(&arg) {
        let v = arg.parse().context("[I] re_raw f64 parse")?;
        if v == 0. {
            bail!("probability cannot be zero");
        }
        return Ok(Raw(v));
    }
    let re_percent =
        Regex::new(r"^(?P<v>[0-9]*(?:\.[0-9]+)?)%$").context("[I] re_percent build re")?;
    if let Some(caps) = re_percent.captures(&arg) {
        let v_str = caps
            .name("v")
            .ok_or(anyhow!("[I] re_percent capture"))?
            .as_str();
        if v_str.is_empty() {
            bail!("could not parse argument");
        }
        let v = v_str.parse().context("[I] re_percent f64 parse")?;
        if v == 0. {
            bail!("probability cannot be zero");
        }
        if v >= 100. {
            bail!("probability must be less than one");
        }
        return Ok(Percent(v));
    }
    let re_one_out_of = Regex::new(r"^[0-9]+$").context("[I] re_one_out_of build re")?;
    if re_one_out_of.is_match(&arg) {
        let v = arg.parse().context("[I] re_one_out_of f64 parse")?;
        if v == 0. {
            bail!("probability cannot be zero");
        }
        if v == 1. {
            bail!("probability must be less than one");
        }
        return Ok(OneOutOf(v));
    }
    let re_ratio = Regex::new(r"^(?P<n>[0-9]+)/(?P<d>[0-9]+)$").context("[I] re_ratio build re")?;
    if let Some(caps) = re_ratio.captures(&arg) {
        let n = caps
            .name("n")
            .ok_or(anyhow!("[I] re_ratio <n> capture access"))?
            .as_str()
            .parse()
            .context("[I] re_ratio <n> f64 parse")?;
        if n == 0. {
            bail!("probability cannot be zero");
        }
        let d = caps
            .name("d")
            .ok_or(anyhow!("[I] re_ratio <d> capture access"))?
            .as_str()
            .parse()
            .context("[I] re_ratio <d> f64 parse")?;
        if d == 0. {
            bail!("division by zero");
        }
        if n >= d {
            bail!("probability must be less than one");
        }
        return Ok(Ratio(n, d));
    }
    bail!("could not parse argument");
}
