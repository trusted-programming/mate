// run-rustfix
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;

fn main() {}

impl PartialOrd for Mode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Mode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let me = match self {
            Mode::FrontEnd => 0,
            Mode::Any => 1,
            Mode::BackGround => 2,
        };
        let other = match other {
            Mode::FrontEnd => 0,
            Mode::Any => 1,
            Mode::BackGround => 2,
        };
        me.cmp(&other)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct QosChange {
    pub(crate) task_id: u32,
    pub(crate) new_qos: Qos,
}

impl QosChange {
    fn new(task_id: u32, new_qos: Qos) -> Self {
        Self { task_id, new_qos }
    }
}

impl QosCase {
    fn new(uid: u64, task_id: u32, mode: Mode, priority: u32) -> Self {
        Self {
            uid,
            task_id,
            mode,
            priority,
            qos: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct QosQueue {
    high_qos_max: usize,

    foreground_high_qos_cases: Vec<QosCase>,
    // bool for sorted
    foreground_low_qos_cases: HashMap<u64, SortQueue>,

    background_high_qos_cases: Vec<QosCase>,

    // bool for sorted
    background_low_qos_cases: HashMap<u64, SortQueue>,

    tasks: HashSet<u32>,

    app_state_map: HashMap<u64, ApplicationState>,
    app_high_qos_count: HashMap<u64, usize>,
}

#[derive(Debug)]
struct SortQueue {
    cases: Vec<QosCase>,
    sorted: bool,
}

impl SortQueue {
    fn push(&mut self, case: QosCase) {
        self.cases.push(case);
        self.sorted = false;
    }

    fn pop_highest_qos(&mut self) -> Option<QosCase> {
        if !self.sorted {
            self.cases.sort_by(|me, other| {
                let res = other.mode.cmp(&me.mode);
                if res == Ordering::Equal {
                    other.priority.cmp(&me.priority)
                } else {
                    res
                }
            });
            self.sorted = true;
        }
        self.cases.pop()
    }

    fn len(&self) -> usize {
        self.cases.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn remove_by_id(&mut self, task_id: u32) {
        for i in 0..self.cases.len() {
            if self.cases[i].task_id == task_id {
                self.cases.remove(i);
                break;
            }
        }
    }
}

#[derive(Debug, Hash, Clone)]
pub(crate) struct QosCase {
    uid: u64,
    task_id: u32,
    mode: Mode,
    priority: u32,
    qos: Option<Qos>,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, PartialOrd, Ord)]
pub(crate) enum ApplicationState {
    Foreground = 2,
    Background = 4,
    Terminated = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub(crate) enum Mode {
    BackGround = 0,
    FrontEnd,
    Any,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum Qos {
    High,
    Low,
}

impl QosQueue {
    fn pop_high_qos(&mut self, state: ApplicationState) -> Option<QosCase> {
        let high_qos_cases = match state {
            ApplicationState::Foreground => &mut self.foreground_high_qos_cases,
            ApplicationState::Background => &mut self.background_high_qos_cases,
            ApplicationState::Terminated => unreachable!(),
        };

        high_qos_cases.sort_by(|me, other| {
            if me.uid == other.uid {
                let res = me.mode.cmp(&other.mode);
                if res == Ordering::Equal {
                    me.priority.cmp(&other.priority)
                } else {
                    res
                }
            } else {
                self.app_high_qos_count
                    .get(&me.uid)
                    .unwrap()
                    .cmp(self.app_high_qos_count.get(&other.uid).unwrap())
            }
        });
        high_qos_cases.pop()
    }
    fn push_high_qos(&mut self, case: QosCase, state: ApplicationState) {
        dbg!("Qos task {} push to {:?} High Qos", case.task_id, state);
        match state {
            ApplicationState::Foreground => {
                self.foreground_high_qos_cases.push(case);
            }
            ApplicationState::Background => {
                self.background_high_qos_cases.push(case);
            }
            ApplicationState::Terminated => unreachable!(),
        }
    }
    fn move_one_high_qos_to_low(
        &mut self,
        qos_changes: &mut Vec<QosChange>,
        state: ApplicationState,
    ) {
        let mut down_grade_case = self.pop_high_qos(state).unwrap();
        change_qos(
            &mut self.app_high_qos_count,
            qos_changes,
            &mut down_grade_case,
            Qos::Low,
        );
        self.push_low_qos(down_grade_case, state);
    }
    fn contest_insert(
        &mut self,
        qos_changes: &mut Vec<QosChange>,
        mut case: QosCase,
        state: ApplicationState,
    ) {
        if *self.app_high_qos_count.get(&case.uid).unwrap() == 0
            && (state == ApplicationState::Foreground || !self.background_high_qos_cases.is_empty())
        {
            self.move_one_high_qos_to_low(qos_changes, state);

            change_qos(
                &mut self.app_high_qos_count,
                qos_changes,
                &mut case,
                Qos::High,
            );
            self.push_high_qos(case, state);
            return;
        }

        let high_qos_cases = match state {
            ApplicationState::Foreground => &mut self.foreground_high_qos_cases,
            ApplicationState::Background => &mut self.background_high_qos_cases,
            ApplicationState::Terminated => unreachable!(),
        };

        let mut down_grade_case = &case;
        let mut swap_case_index_opt = None;
        (high_qos_cases.iter().enumerate())
            .filter(|(i, swap_case)| {
                down_grade_case.uid == swap_case.uid
                    && (down_grade_case.mode < swap_case.mode
                        || down_grade_case.priority < swap_case.priority)
            })
            .for_each(|(i, swap_case)| {
                down_grade_case = swap_case;
                swap_case_index_opt = Some(i)
            });

        if let Some(i) = swap_case_index_opt {
            change_qos(
                &mut self.app_high_qos_count,
                qos_changes,
                &mut case,
                Qos::High,
            );
            mem::swap(&mut case, high_qos_cases.get_mut(i).unwrap());
            change_qos(
                &mut self.app_high_qos_count,
                qos_changes,
                &mut case,
                Qos::Low,
            );
            self.push_low_qos(case, state);
        } else {
            change_qos(
                &mut self.app_high_qos_count,
                qos_changes,
                &mut case,
                Qos::Low,
            );
            self.push_low_qos(case, state);
        }
    }
    fn push_low_qos(&mut self, case: QosCase, state: ApplicationState) {
        dbg!("Qos task {} push to {:?} Low Qos", case.task_id, state);

        let low_qos_cases = match state {
            ApplicationState::Foreground => &mut self.foreground_low_qos_cases,
            ApplicationState::Background => &mut self.background_low_qos_cases,
            ApplicationState::Terminated => unreachable!(),
        };

        match low_qos_cases.get_mut(&case.uid) {
            Some(cases) => {
                cases.push(case);
            }
            None => {
                let mut cases = Vec::new();
                let uid = case.uid;
                cases.push(case);

                low_qos_cases.insert(
                    uid,
                    SortQueue {
                        cases,
                        sorted: true,
                    },
                );
            }
        }
    }
}
fn change_qos(
    app_high_qos_count: &mut HashMap<u64, usize>,
    qos_changes: &mut Vec<QosChange>,
    case: &mut QosCase,
    new_qos: Qos,
) {
    match case.qos.take() {
        Some(old_qos) => {
            if old_qos == new_qos {
                dbg!(
                    "Qos change task {} qos {:?} with the same",
                    case.task_id,
                    old_qos
                );
                return;
            }
            let count = app_high_qos_count.get_mut(&case.uid).unwrap();
            match new_qos {
                Qos::High => *count += 1,
                Qos::Low => *count -= 1,
            }

            case.qos = Some(new_qos.clone());
            dbg!("Qos task {} change to {:?} Qos", case.task_id, &new_qos);
            qos_changes.push(QosChange::new(case.task_id, new_qos));
        }
        None => {
            if new_qos == Qos::High {
                let count = app_high_qos_count.get_mut(&case.uid).unwrap();
                *count += 1;
            }
            case.qos = Some(new_qos.clone());
            dbg!("Qos task {} change to {:?} Qos", case.task_id, &new_qos);
            qos_changes.push(QosChange::new(case.task_id, new_qos));
        }
    }
}
