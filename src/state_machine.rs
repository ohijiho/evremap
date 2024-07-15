use crate::mapping::*;
use crate::util::vec_into_sorted;
use evdev_rs::{InputEvent, TimeVal};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
enum KeyEventType {
    Release,
    Press,
    Repeat,
    Unknown(i32),
}

impl KeyEventType {
    fn from_value(value: i32) -> Self {
        match value {
            0 => KeyEventType::Release,
            1 => KeyEventType::Press,
            2 => KeyEventType::Repeat,
            _ => KeyEventType::Unknown(value),
        }
    }

    fn value(&self) -> i32 {
        match self {
            Self::Release => 0,
            Self::Press => 1,
            Self::Repeat => 2,
            Self::Unknown(n) => *n,
        }
    }
}

pub struct StateMachine {
    key_map: HashMap<KeyCode, KeyEntry>,

    input_state: HashMap<KeyCode, HashSet<KeyCode>>,
    output_state: HashMap<KeyCode, HashSet<KeyCode>>,
}

#[derive(Debug)]
struct CondBits {
    buf: Box<[u8]>,
    unsatisfied: usize,
}

#[derive(Clone, Debug)]
struct CondBitRef {
    cb: Rc<RefCell<CondBits>>,
    i: usize,
}

#[derive(Debug)]
struct RemapRule {
    cond_bits: Rc<RefCell<CondBits>>,
    mappings: Box<[(KeyCode, Box<[KeyCode]>)]>,
}

#[derive(Debug)]
struct KeyEntry {
    cond_set: Vec<CondBitRef>,
    cond_unset: Vec<CondBitRef>,
    trigger: Vec<Rc<RemapRule>>,
}

impl CondBits {
    fn new(unsatisfied: usize, cap: usize) -> Self {
        let mut buf = vec![0u8; (cap + 7) >> 3];
        for i in 0..(unsatisfied >> 3) {
            buf[i] = 0xff;
        }
        if unsatisfied & 7 != 0 {
            buf[unsatisfied >> 3] = (1u8 << (unsatisfied & 7) as u8) - 1;
        }
        Self {
            buf: buf.into(),
            unsatisfied,
        }
    }

    fn set(&mut self, index: usize, value: bool) {
        let i = index >> 3;
        let s = index & 7;
        let old = self.buf[i] & (1 << s) != 0;
        if value == old {
            return;
        }
        if value {
            self.unsatisfied += 1;
            self.buf[i] |= 1u8 << s;
        } else {
            self.unsatisfied -= 1;
            self.buf[i] &= !(1u8 << s);
        }
    }
}
impl CondBitRef {
    fn set(&self, value: bool) {
        self.cb.borrow_mut().set(self.i, value);
    }
}

impl StateMachine {
    pub fn new(mappings: &[Mapping]) -> Self {
        let remaps: Vec<((Vec<_>, Vec<_>), _, _)> = mappings
            .iter()
            .filter_map(|x| match x {
                Mapping::Remap {
                    cond,
                    except,
                    when,
                    mappings,
                } => Some((
                    (
                        vec_into_sorted(cond.iter().map(Clone::clone).collect()),
                        vec_into_sorted(except.iter().map(Clone::clone).collect()),
                    ),
                    when,
                    mappings.as_ref(),
                )),
            })
            .collect();

        let cond_vec: Vec<_> = remaps
            .iter()
            .map(|((a, b), ..)| (a.clone(), b.clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let cond_bits: Vec<_> = cond_vec
            .iter()
            .map(|(a, b)| Rc::from(RefCell::new(CondBits::new(a.len(), a.len() + b.len()))))
            .collect();
        let cond_tbl: HashMap<_, _> = cond_vec
            .into_iter()
            .enumerate()
            .map(|(i, x)| (x, cond_bits[i].clone()))
            .collect();

        let remap_rules: Box<_> = remaps
            .iter()
            .map(|(cond, _, mappings)| {
                Rc::from(RemapRule {
                    cond_bits: cond_tbl[cond].clone(),
                    mappings: mappings.iter().map(Clone::clone).collect(),
                })
            })
            .collect();
        let remaps: Vec<_> = remaps
            .into_iter()
            .enumerate()
            .map(|(i, (cond, when, ..))| (cond, when, &remap_rules[i]))
            .collect();

        let mut key_map = HashMap::<KeyCode, KeyEntry>::new();
        let kedef = || KeyEntry {
            cond_set: vec![],
            cond_unset: vec![],
            trigger: vec![],
        };

        for ((a, b), v) in cond_tbl {
            for (i, x) in a.iter().enumerate() {
                let e = key_map.entry(*x).or_insert_with(kedef);
                e.cond_set.push(CondBitRef { cb: v.clone(), i })
            }
            for (i, x) in b.iter().enumerate() {
                let e = key_map.entry(*x).or_insert_with(kedef);
                e.cond_unset.push(CondBitRef {
                    cb: v.clone(),
                    i: a.len() + i,
                })
            }
        }

        for (_, when, rule) in remaps {
            for x in when {
                let e = key_map.entry(*x).or_insert_with(kedef);
                e.trigger.push(rule.clone());
            }
        }

        Self {
            key_map,
            input_state: HashMap::new(),
            output_state: HashMap::new(),
        }
    }

    pub fn send(&mut self, event: &InputEvent) -> Vec<InputEvent> {
        if let EventCode::EV_KEY(ref key) = event.event_code {
            log::trace!("IN {:?}", event);
            self.update_with_event(&event, key.clone())
        } else {
            log::trace!("PASSTHRU {:?}", event);
            vec![event.clone()]
        }
    }

    fn update_with_event(&mut self, event: &InputEvent, code: KeyCode) -> Vec<InputEvent> {
        let event_type = KeyEventType::from_value(event.value);

        match event_type {
            KeyEventType::Press => {
                if let Some(e) = self.key_map.get(&code) {
                    for c in &e.cond_set {
                        c.set(false);
                    }
                }
            }
            KeyEventType::Release => {
                if let Some(e) = self.key_map.get(&code) {
                    for c in &e.cond_unset {
                        c.set(true);
                    }
                }
            }
            _ => {}
        }

        match event_type {
            KeyEventType::Release | KeyEventType::Press => {
                let is_release = match event_type {
                    KeyEventType::Release => true,
                    KeyEventType::Press => false,
                    _ => unreachable!(),
                };
                let (release, press) = self.remap(code, is_release);
                if is_release && !press.is_empty() {
                    return vec![event.clone(), make_sync_event(&event.time)];
                }
                let (release, press) = self.update(
                    &release,
                    &press
                        .iter()
                        .map(|(k, v)| (*k, v.as_slice()))
                        .collect::<Vec<_>>(),
                );

                release
                    .into_iter()
                    .map(|k| make_event(k.clone(), &event.time, KeyEventType::Release))
                    .chain(
                        press
                            .into_iter()
                            .map(|k| make_event(k.clone(), &event.time, KeyEventType::Press)),
                    )
                    .chain(vec![make_sync_event(&event.time)].into_iter())
                    .collect()
            }
            KeyEventType::Repeat => {
                if let Some(target_keys) = self.input_state.get(&code) {
                    target_keys
                        .iter()
                        .map(|k| make_event(k.clone(), &event.time, KeyEventType::Repeat))
                        .chain(vec![make_sync_event(&event.time)].into_iter())
                        .collect()
                } else {
                    log::warn!("Repeat of unpressed key {code:?}");
                    vec![event.clone()]
                }
            }
            KeyEventType::Unknown(x) => {
                log::warn!("Unknown event type {x} of key {code:?}");
                vec![event.clone()]
            }
        }
    }

    fn remap(&self, code: KeyCode, release: bool) -> (Vec<KeyCode>, Vec<(KeyCode, Vec<KeyCode>)>) {
        if release {
            if self.input_state.contains_key(&code) {
                (vec![code], vec![])
            } else {
                log::warn!("Release of unpressed key {code:?}");
                if self.output_state.contains_key(&code) {
                    (vec![], vec![])
                } else {
                    (vec![code], vec![(code, vec![code])])
                    // this is handled as a special case
                }
            }
        } else {
            if let Some(e) = self.key_map.get(&code) {
                let mut mappings = HashMap::new(); // TODO: deterministic order

                for rule in &e.trigger {
                    if rule.cond_bits.borrow().unsatisfied == 0 {
                        for (src, dst) in rule.mappings.iter() {
                            if let Entry::Vacant(e) = mappings.entry(*src) {
                                e.insert(&**dst);
                            } else {
                                log::warn!("Ambiguous rules made active simultaneously: {src:?} -> ([{:?}] vs [{:?}])", mappings[src], &**dst);
                            }
                        }
                    }
                }

                let mut release = Vec::new();
                let mut press = Vec::new();

                for (src, dst) in &mappings {
                    if *src == code {
                        continue;
                    }

                    if self.input_state.contains_key(src) {
                        release.push(*src);
                        press.push((*src, dst.iter().map(Clone::clone).collect()));
                    }
                }

                // These keys are pressed after any other keys.
                if let Some(dst) = mappings.get(&code) {
                    if self.input_state.contains_key(&code) {
                        log::warn!("Double pressed key {code:?}");
                        release.push(code);
                    }
                    press.push((code, dst.iter().map(Clone::clone).collect()));
                } else {
                    press.push((code, vec![code]));
                }

                (release, press)
            } else {
                if self.input_state.contains_key(&code) {
                    log::warn!("Double pressed key {code:?}");
                    (vec![code], vec![(code, vec![code])])
                } else {
                    (vec![], vec![(code, vec![code])])
                }
            }
        }
    }

    fn update(
        &mut self,
        release: &[KeyCode],
        press: &[(KeyCode, &[KeyCode])],
    ) -> (Vec<KeyCode>, Vec<KeyCode>) {
        let mut released = Vec::new();
        for code in release {
            for x in self.input_state.remove(code).unwrap() {
                let y = self.output_state.get_mut(&x).unwrap();
                y.remove(&code);
                if y.is_empty() {
                    self.output_state.remove(&x);
                    released.push(x);
                }
            }
        }

        let mut pressed = Vec::new();
        for (code, dst) in press {
            let set = dst.iter().map(Clone::clone).collect();
            self.input_state.insert(*code, set);
            for x in dst.iter() {
                match self.output_state.entry(*x) {
                    Entry::Vacant(e) => {
                        e.insert(vec![*code].into_iter().collect());
                        pressed.push(*x);
                    }
                    Entry::Occupied(e) => {
                        e.into_mut().insert(*code);
                    }
                }
            }
        }

        let pressed_set: HashSet<_> = pressed.iter().collect();
        (
            released
                .into_iter()
                .filter(|x| !pressed_set.contains(x))
                .collect(),
            pressed,
        )
    }
}

fn make_sync_event(time: &TimeVal) -> InputEvent {
    InputEvent::new(
        time,
        &EventCode::EV_SYN(evdev_rs::enums::EV_SYN::SYN_REPORT),
        0,
    )
}

fn make_event(key: KeyCode, time: &TimeVal, event_type: KeyEventType) -> InputEvent {
    InputEvent::new(time, &EventCode::EV_KEY(key), event_type.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyCode::*;
    use KeyEventType::*;

    fn ke(key: KeyCode, sec: i64, event_type: KeyEventType) -> InputEvent {
        make_event(key, &TimeVal::new(sec, 0), event_type)
    }

    fn se(sec: i64) -> InputEvent {
        make_sync_event(&TimeVal::new(sec, 0))
    }

    fn remap(
        cond: Vec<KeyCode>,
        except: Vec<KeyCode>,
        when: Vec<KeyCode>,
        mappings: Vec<(KeyCode, Vec<KeyCode>)>,
    ) -> Mapping {
        Mapping::Remap {
            cond: cond.into_iter().collect(),
            except: except.into_iter().collect(),
            when: when.into_iter().collect(),
            mappings: mappings
                .into_iter()
                .map(|(k, v)| (k, v.into_boxed_slice()))
                .collect(),
        }
    }

    #[test]
    fn test_tbl() {
        let mut builder = pretty_env_logger::formatted_timed_builder();
        if let Ok(s) = std::env::var("EVREMAP_LOG") {
            builder.parse_filters(&s);
        } else {
            builder.filter(None, log::LevelFilter::Info);
        }
        builder.init();
        log::info!("info logsaoisejroaiej");

        let tcs: Vec<(_, _, Vec<(InputEvent, Vec<InputEvent>)>)> = vec![
            (
                "case 1",
                vec![],
                vec![
                    (ke(KEY_1, 10, Press), vec![ke(KEY_1, 10, Press), se(10)]),
                    (ke(KEY_1, 10, Release), vec![ke(KEY_1, 10, Release), se(10)]),
                ],
            ),
            (
                "case 2",
                vec![
                    remap(
                        vec![],
                        vec![],
                        vec![KEY_LEFTMETA],
                        vec![(KEY_LEFTMETA, vec![])],
                    ),
                    remap(
                        vec![KEY_LEFTMETA],
                        vec![],
                        vec![KEY_C, KEY_V],
                        vec![(KEY_LEFTMETA, vec![KEY_LEFTCTRL])],
                    ),
                    remap(
                        vec![KEY_LEFTMETA],
                        vec![],
                        vec![KEY_TAB, KEY_GRAVE],
                        vec![(KEY_LEFTMETA, vec![KEY_LEFTMETA])],
                    ),
                ],
                vec![
                    (ke(KEY_LEFTMETA, 10, Press), vec![se(10)]),
                    (
                        ke(KEY_C, 11, Press),
                        vec![ke(KEY_LEFTCTRL, 11, Press), ke(KEY_C, 11, Press), se(11)],
                    ),
                    (ke(KEY_C, 12, Release), vec![ke(KEY_C, 12, Release), se(12)]),
                    (
                        ke(KEY_LEFTMETA, 13, Release),
                        vec![ke(KEY_LEFTCTRL, 13, Release), se(13)],
                    ),
                ],
            ),
        ];
        for (name, mappings, calls) in tcs {
            log::trace!("test case {name} begin");
            let mut sm = StateMachine::new(&mappings);
            for (input, output) in calls {
                assert_eq!((name, &input, sm.send(&input)), (name, &input, output));
            }
            log::trace!("test case {name} end");
        }
    }
}
