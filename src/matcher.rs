extern crate clap;
extern crate crossbeam;

use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::rules;

pub fn worker_logic(
    rules: Vec<rules::Rule>,
    wordlist: &Vec<Vec<u8>>,
    aclear: &HashMap<Vec<u8>, Vec<(Vec<u8>, Vec<u8>, u64)>>,
    cutoff: usize,
) -> HashMap<Vec<rules::Rule>, BTreeSet<u64>> {
    let mut hits: HashMap<Vec<rules::Rule>, BTreeSet<u64>> = HashMap::new();
    for word in wordlist.iter() {
        match rules::mutate(&word, &rules) {
            None => (),
            Some(mutated) => match aclear.get(&mutated) {
                None => (),
                Some(matches) => {
                    for (prefix, suffix, nth) in matches {
                        use rules::CommandRule::InsertString;
                        use rules::Numerical::{Infinite, Val};
                        use rules::Rule::Command;
                        let mut currule = rules.clone();
                        if prefix.len() > 0 {
                            currule.push(Command(InsertString(Val(0), prefix.clone())));
                        }
                        if suffix.len() > 0 {
                            currule.push(Command(InsertString(Infinite, suffix.clone())));
                        }
                        hits.entry(currule)
                            .and_modify(|hs| {
                                hs.insert(*nth);
                            })
                            .or_insert_with(|| {
                                let mut o = BTreeSet::new();
                                o.insert(*nth);
                                o
                            });
                    }
                }
            },
        };
    }
    hits.retain(|_, st| st.len() >= cutoff);
    return hits;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleartexts;
    use crate::rules::CommandRule::*;
    use crate::rules::Numerical::*;
    use crate::rules::Rule::*;

    fn conv(i: &str) -> Vec<u8> {
        i.as_bytes().to_vec()
    }

    #[test]
    fn test1() {
        let wordlist = vec![conv("ABC"), conv("DEF"), conv("ABCDEF"), conv("hal9000")];
        let mut clears = HashMap::new();
        cleartexts::process_line(&mut clears, 0, &conv("ABC12"), 3);
        cleartexts::process_line(&mut clears, 1, &conv("DEF12"), 3);
        cleartexts::process_line(&mut clears, 2, &conv("ABCDE"), 3);
        cleartexts::process_line(&mut clears, 3, &conv("CBA"), 3);
        cleartexts::process_line(&mut clears, 4, &conv("0009lah"), 3);

        let mut s01 = BTreeSet::new();
        s01.insert(0);
        s01.insert(1);
        let mut s2 = BTreeSet::new();
        s2.insert(2);
        let mut s34 = BTreeSet::new();
        s34.insert(3);
        s34.insert(4);

        let res_noop = worker_logic(vec![], &wordlist, &clears, 1);
        let mut expected = HashMap::new();
        expected.insert(
            vec![Command(InsertString(Infinite, conv("12")))],
            s01.clone(),
        );
        expected.insert(
            vec![Command(InsertString(Infinite, conv("DE")))],
            s2.clone(),
        );

        expected.clear();
        let cmd_truncate3 = Command(Truncate(Val(3)));
        let res_truncate3 = worker_logic(vec![cmd_truncate3.clone()], &wordlist, &clears, 1);
        expected.insert(
            vec![
                cmd_truncate3.clone(),
                Command(InsertString(Infinite, conv("12"))),
            ],
            s01.clone(),
        );
        expected.insert(
            vec![cmd_truncate3, Command(InsertString(Infinite, conv("DE")))],
            s2.clone(),
        );
        assert_eq!(res_truncate3, expected);

        expected.clear();
        let cmd_reverse = Command(Reverse);
        let res_reverse = worker_logic(vec![cmd_reverse.clone()], &wordlist, &clears, 1);
        expected.insert(vec![cmd_reverse], s34.clone());
        assert_eq!(res_reverse, expected);
    }
}