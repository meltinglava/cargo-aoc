use petgraph::Direction;
use petgraph::Graph;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;
use std::string::FromUtf8Error;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Step(u8);

impl Step {
    fn duration(self, base_time: u32) -> u32 {
        u32::from(self.0 - b'A' + 1) + base_time
    }
}

impl Debug for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0 as char)
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0 as char)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Instruction {
    required: Step,
    step: Step,
}

impl FromStr for Instruction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.len() != "Step C must be finished before step A can begin.".len() {
            return Err("Wrong length");
        }
        let s = s.as_bytes();
        Ok(Instruction {
            required: Step(s[5]),
            step: Step(s[36]),
        })
    }
}

fn parse(input: &str) -> Result<Graph<Step, ()>, &'static str> {
    use petgraph::graphmap::DiGraphMap;

    let mut graph = DiGraphMap::new();

    for l in input.lines() {
        let instruction: Instruction = l.parse()?;

        graph.add_node(instruction.required);
        graph.add_node(instruction.step);

        graph.add_edge(instruction.required, instruction.step, ());
    }

    Ok(graph.into_graph())
}

fn part1(graph: &Graph<Step, ()>) -> Result<String, FromUtf8Error> {
    let mut remaining = graph.clone();

    let mut seq = Vec::with_capacity(graph.node_count());

    loop {
        if let Some(i) = remaining
            .externals(Direction::Incoming)
            .min_by_key(|&i| remaining[i])
        {
            seq.push(remaining.remove_node(i).unwrap().0);
        } else {
            break String::from_utf8(seq);
        }
    }
}

mod day7_part1 {
    use super::*;
    use crate::{Day7Part1, Factory};
    use aoc_runner::{ArcStr, Runner};
    use std::borrow::Borrow;
    use std::error::Error;
    use std::fmt::Display;
    use std::marker::PhantomData;
    impl Day7Part1 for Factory {
        fn day7_part1(input: ArcStr) -> Result<Box<dyn Runner>, Box<dyn Error>> {
            Ok(Box::new(RunnerStruct::try_gen(input)?))
        }
    }
    pub struct RunnerStruct {
        input: Graph<Step, ()>,
        output: PhantomData<String>,
    }
    impl Runner for RunnerStruct {
        fn gen(input: ArcStr) -> Self {
            Self::try_gen(input).expect("failed to generate input")
        }
        fn try_gen(input: ArcStr) -> Result<Self, Box<dyn Error>> {
            Ok(RunnerStruct {
                input: parse(input.borrow())?,
                output: PhantomData,
            })
        }
        fn run(&self) -> Box<dyn Display> {
            self.try_run().expect("failed to run")
        }
        fn try_run(&self) -> Result<Box<dyn Display>, Box<dyn Error>> {
            Ok(Box::new(part1(self.input.borrow())?))
        }
        fn bench(&self, black_box: fn(&dyn Display)) {
            black_box(&part1(self.input.borrow()).unwrap())
        }
    }
}

fn part2(graph: &Graph<Step, ()>) -> u32 {
    part2_internal(graph, 5, 60)
}

mod day7_part2 {
    use super::*;
    use crate::{Day7Part2, Factory};
    use aoc_runner::{ArcStr, Runner};
    use std::borrow::Borrow;
    use std::error::Error;
    use std::fmt::Display;
    use std::marker::PhantomData;
    impl Day7Part2 for Factory {
        fn day7_part2(input: ArcStr) -> Result<Box<dyn Runner>, Box<dyn Error>> {
            Ok(Box::new(RunnerStruct::try_gen(input)?))
        }
    }
    pub struct RunnerStruct {
        input: Graph<Step, ()>,
        output: PhantomData<u32>,
    }
    impl Runner for RunnerStruct {
        fn gen(input: ArcStr) -> Self {
            Self::try_gen(input).expect("failed to generate input")
        }
        fn try_gen(input: ArcStr) -> Result<Self, Box<dyn Error>> {
            Ok(RunnerStruct {
                input: parse(input.borrow())?,
                output: PhantomData,
            })
        }
        fn run(&self) -> Box<dyn Display> {
            Box::new(part2(self.input.borrow()))
        }
        fn bench(&self, black_box: fn(&dyn Display)) {
            black_box(&part2(self.input.borrow()))
        }
    }
}

fn part2_internal(graph: &Graph<Step, ()>, nb_worker: usize, base_time: u32) -> u32 {
    let mut remaining = graph.clone();
    let mut workers = vec![(None, 0); nb_worker];
    let mut started = Vec::with_capacity(remaining.node_count());

    loop {
        let &mut (ref mut job, ref mut time) = workers
            .iter_mut()
            .min_by(|a, b| {
                a.1.cmp(&b.1).then_with(|| match (a.0, b.0) {
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    _ => Ordering::Equal,
                })
            })
            .unwrap();

        if let Some(step) = job.take() {
            let i = remaining
                .externals(Direction::Incoming)
                .find(|&i| remaining[i] == step)
                .unwrap();
            remaining.remove_node(i);
        };

        if let Some(step) = remaining
            .externals(Direction::Incoming)
            .map(|i| remaining[i])
            .filter(|step| !started.contains(step))
            .min()
        {
            started.push(step);
            *job = Some(step);
            *time += step.duration(base_time);
        } else if remaining.node_count() == 0 {
            break workers.into_iter().max().unwrap().1;
        } else {
            *time += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "Step C must be finished before step A can begin.
Step C must be finished before step F can begin.
Step A must be finished before step B can begin.
Step A must be finished before step D can begin.
Step B must be finished before step E can begin.
Step D must be finished before step E can begin.
Step F must be finished before step E can begin.";

    #[test]
    fn instructions() {
        assert_eq!(
            "Step C must be finished before step A can begin.".parse(),
            Ok(Instruction {
                required: Step(b'C'),
                step: Step(b'A'),
            })
        );

        assert_eq!(Step(b'A').duration(0), 1);
        assert_eq!(Step(b'Z').duration(0), 26);
    }

    #[test]
    fn part1_example() {
        let graph = parse(INPUT).unwrap();

        assert_eq!(part1(&graph).unwrap(), "CABDFE".to_string());
    }

    #[test]
    fn part2_example() {
        let graph = parse(INPUT).unwrap();

        assert_eq!(part2_internal(&graph, 2, 0), 15);
    }
}
