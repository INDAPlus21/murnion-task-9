use std::fs;
use std::collections;
use rand::Rng;
use rand::rngs;
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use chrono::prelude::*;

#[derive(Clone, PartialEq)]
enum PrefixType {
    POLITE_STRONG,
    POLITE,
    DEMANDING,
    DEMANDING_STRONG,
}

impl PrefixType {
    fn find(code: [bool; 2]) -> PrefixType {
        match code {
            [false, false] => PrefixType::DEMANDING_STRONG,
            [false, true] => PrefixType::DEMANDING,
            [true, false] => PrefixType::POLITE,
            [true, true] => PrefixType::POLITE_STRONG,
            _ => panic!()
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
enum Mood {
    Bored, //no changes to mood scores as they are by default.
    Happy, //social credit starts high, higher tolerances.
    Sick, //social credit starts low, tolerance is low on the positive end.
    Maniacal, //social credit cannot increase. Close to tolerance is always positive for DEMAND.
    Angry, //social credit decreases much faster, and the tolerance is lower on the negative end.
    Annoyed, //irritation increases twice as fast.
    Lovestruck, //social credit increases much faster, and tolerance is all around high.
    Confused, //social credit increases instead of decreasing, and vice-versa.
}

#[derive(Clone, PartialEq)]
enum OperationType {
    INCREMENT,
    TO,
    ACCESS,
    LOOP,
    BRANCH_IF_GREATER,
    BRANCH_IF_ZERO,
    BRANCH_IF_EQUAL,
    JUMP,
}

impl OperationType {
    fn find(code: [bool; 3]) -> OperationType {
        match code {
            [false, false, false] => OperationType::INCREMENT,
            [false, false, true] => OperationType::TO,
            [false, true, false] => OperationType::ACCESS,
            [false, true, true] => OperationType::LOOP,
            [true, false, false] => OperationType::BRANCH_IF_GREATER,
            [true, false, true] => OperationType::BRANCH_IF_ZERO,
            [true, true, false] => OperationType::BRANCH_IF_EQUAL,
            [true, true, true] => OperationType::JUMP,
            _ => panic!()
        }
    }
}

fn convert_to_instruction(_byte: u8) -> (PrefixType, OperationType, [bool; 3]) {
    let mut byte = _byte;
    let base: u8 = 2;
    let mut bool_array: [bool; 8] = [false; 8];
    for i in 0..8 {
        if byte >= base.pow(8 - i) {
            byte -= base.pow(8 - i);
            bool_array[i as usize] = true;
        }
    }

    let operation_type = OperationType::find([bool_array[0], bool_array[1], bool_array[2]]);
    let prefix_type = PrefixType::find([bool_array[3], bool_array[4]]);
    let specifics = [bool_array[5], bool_array[6], bool_array[7]];

    (prefix_type, operation_type, specifics)
}

fn to_specifics(string: &str) -> [bool; 3] {
    let temp1 = string.split_at(1).0.parse::<bool>().unwrap();
    let temp2 = string.split_at(1).1.split_at(1).0.parse::<bool>().unwrap();
    let temp3 = string.split_at(1).1.split_at(1).1.parse::<bool>().unwrap();

    let specifics = [temp1, temp2, temp3];
    specifics
}

fn print_error(err_msg: &str) {
    println!("[ERROR] {}", err_msg);
}

fn generate_mood(mut irritation: i32, mut credit: i32, mut mood: Mood, mut small: i32, 
    mut small_close: i32, mut medium: i32, mut medium_close: i32, mut large: i32, mut large_close: i32,
    mut polite_change: i32, mut polite_strong_change: i32, mut demand_change: i32, mut demand_strong_change: i32,
    mut irritation_change: i32, mut irritation_decay: i32) {
    let time = chrono::offset::Utc::now();
    let hours = time.time().hour();
    let days = time.date().ordinal();
    let seed = hours * days;
    let mut rng: rngs::StdRng = rand::SeedableRng::seed_from_u64(seed as u64);

    match (rng.gen::<u64>() % 8) {
        0 => {
            mood = Mood::Bored;
        },
        1 => {
            mood = Mood::Happy;
            credit = 50;
            small = 75;
            medium = 100;
            large = 125;
            small_close = 50;
            medium_close = 75;
            large_close = 100;
        },
        2 => {
            mood = Mood::Sick;
            credit = -25;
            polite_strong_change = -7;
            demand_change = -4;
        },
        3 => {
            mood = Mood::Maniacal;
            irritation_decay = 0;
            medium_close = 0;
        },
        4 => {
            mood = Mood::Angry;
            demand_change = -4;
            demand_strong_change = -8;
            polite_strong_change = -4;
            large = 75;
            large_close = 50;
        },
        5 => {
            mood = Mood::Annoyed;
            irritation_change = 8;
        },
        6 => {
            mood = Mood::Lovestruck;
            polite_change = 5;
            polite_strong_change = -2;
            credit = 25;
            small = 100;
            medium = 125;
            large = 150;
            small_close = 50;
            medium_close = 75;
            large_close = 100;
            irritation_decay = -2;
        },
        7 => {
            mood = Mood::Confused;
            polite_change = -2;
            polite_strong_change = -4;
            demand_change = 2;
            demand_strong_change = 5;
        },
        _ => panic!("Generated a number outside of the range, for moods!")
    }
}

pub fn emulate() {
    let mut irritation: i32 = 0;
    let mut last_was_positive = true;
    let mut social_credit: i32 = 0;
    let mut mood = Mood::Bored;
    let mut SMALL_TOLERANCE: i32 = 50;
    let mut SMALL_TOLERANCE_CLOSE: i32 = 25;
    let mut MEDIUM_TOLERANCE: i32 = 75;
    let mut MEDIUM_TOLERANCE_CLOSE: i32 = 50;
    let mut LARGE_TOLERANCE: i32 = 100;
    let mut LARGE_TOLERANCE_CLOSE: i32 = 75;

    let mut polite_social_change: i32 = 2;
    let mut polite_strong_social_change: i32 = -5;
    let mut demanding_social_change: i32 = -2;
    let mut demanding_strong_social_change: i32 = -5;
    let mut irritation_change: i32 = 5;
    let mut irritation_decay: i32 = -1;

    generate_mood(irritation, social_credit, mood, 
                SMALL_TOLERANCE, SMALL_TOLERANCE_CLOSE, MEDIUM_TOLERANCE, MEDIUM_TOLERANCE_CLOSE, LARGE_TOLERANCE, LARGE_TOLERANCE_CLOSE,
                polite_social_change, polite_strong_social_change, demanding_social_change, demanding_strong_social_change,
                irritation_change, irritation_decay);

    let mut polite_registries: (i32, i32) = (0, 0);
    let mut demanding_registries: (i32, i32) = (0, 0);
    let mut stacks = (Vec::<i32>::new(), Vec::<i32>::new());
    let mut loops: (i32, i32) = (0, 0);
    let mut loop_counters: (i32, i32) = (0, 0);
    let mut loop_registries: (i32, i32) = (0, 0);

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 {
        if args[2].clone() == "get_mood" {
            println!("{:?}", mood);
        }
    }
    if args.len() < 2 {
        print_error("A path to a SAL file must be provided!");
    } else {
        let mut commands: Vec<(PrefixType, OperationType, [bool; 3])> = fs::read(args[1].clone()).ok().unwrap()
                                                                                        .into_iter()
                                                                                        .map(|x| convert_to_instruction(x))
                                                                                        .collect();

        for mut i in 0..commands.len() {
            let prefix_type = commands[i].0.clone();
            let operation_type = commands[i].1.clone();
            let specifics = commands[i].2;

            if irritation >= 100 {
                eprintln!("This program is DONE with your wishy-washy attitude.");
                std::thread::sleep(std::time::Duration::from_millis(500));
                panic!();
            }

            match prefix_type {
                PrefixType::POLITE => {
                    if social_credit >= SMALL_TOLERANCE || social_credit <= SMALL_TOLERANCE*-1 {
                        continue;
                    }
                    if social_credit >= SMALL_TOLERANCE_CLOSE || social_credit <= SMALL_TOLERANCE_CLOSE*-1 {
                        eprintln!("\"I guess...\"");
                    }
                    if !last_was_positive {
                        irritation += irritation_change;
                    } else if irritation > 0 {
                        irritation += irritation_decay;
                    }
                    social_credit += polite_social_change;
                },
                PrefixType::POLITE_STRONG => {
                    if social_credit >= LARGE_TOLERANCE || social_credit <= LARGE_TOLERANCE*-1 {
                        eprintln!("The program got tired of your snivelling attitude and left...");
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        panic!();
                    }
                    if social_credit >= LARGE_TOLERANCE_CLOSE || social_credit <= LARGE_TOLERANCE_CLOSE*-1 {
                        std::thread::sleep(std::time::Duration::from_millis(1500));
                    }
                    if last_was_positive {
                        irritation += irritation_change;
                    } else if irritation > 0 {
                        irritation += irritation_decay;
                    }
                    social_credit += polite_strong_social_change;
                },
                PrefixType::DEMANDING => {
                    if social_credit >= MEDIUM_TOLERANCE || social_credit <= MEDIUM_TOLERANCE*-1 {
                        eprintln!("\"Oh, now? Really, now?\"");
                        eprintln!("The program is defiantly doing nothing.");
                        std::thread::sleep(std::time::Duration::from_millis(15000));
                        eprintln!("\"Fine.\"");
                    }
                    if social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= MEDIUM_TOLERANCE_CLOSE*-1 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    if last_was_positive {
                        irritation += irritation_change;
                    } else if irritation > 0 {
                        irritation += irritation_decay;
                    }
                    social_credit += demanding_social_change;
                }
                PrefixType::DEMANDING_STRONG => {
                    if social_credit >= MEDIUM_TOLERANCE || social_credit <= MEDIUM_TOLERANCE*-1 {
                        eprintln!("\"No, you know what, f*** you!\"");
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        panic!();
                    }
                    if last_was_positive {
                        irritation += irritation_change;
                    } else if irritation > 0 {
                        irritation += irritation_decay;
                    }
                    social_credit += demanding_strong_social_change;
                }
                _ => panic!()
            }

            match operation_type {
                OperationType::INCREMENT => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    let mut other_registry = 0;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                        other_registry = selected_registries.0;
                    } else {
                        selected_registry = selected_registries.0;
                        other_registry = selected_registries.1;
                    }

                    let mut value = 1;
                    if specifics[2] {
                        value = other_registry;
                    }
                    if prefix_type == PrefixType::DEMANDING_STRONG && (social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= -1*MEDIUM_TOLERANCE_CLOSE) {
                        value *= 2;
                    }

                    if specifics[1] {
                        selected_registry -= value;
                    } else {
                        selected_registry += value;
                    }

                    if specifics[0] {
                        selected_registries.1 = selected_registry;
                    } else {
                        selected_registries.0 = selected_registry;
                    }

                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        polite_registries = selected_registries;
                    } else {
                        demanding_registries = selected_registries;
                    }
                },
                OperationType::TO => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    let mut selected_stack: &mut Vec<i32>;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                        selected_stack = &mut stacks.1;
                    } else {
                        selected_registry = selected_registries.0;
                        selected_stack = &mut stacks.0;
                    }

                    if specifics[1] {
                        if specifics[2] {
                            let mut temp = selected_stack.pop();
                            if temp == None { temp = Some(0); }
                            selected_registry = temp.unwrap();
                        } else {
                            selected_registry /= 2;
                        }
                    } else {
                        if specifics[2] {
                            selected_stack.push(selected_registry);
                        } else {
                            selected_registry *= 2;
                        }
                    }

                    if specifics[0] {
                        selected_registries.1 = selected_registry;
                    } else {
                        selected_registries.0 = selected_registry;
                    }

                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        polite_registries = selected_registries;
                    } else {
                        demanding_registries = selected_registries;
                    }
                }
                OperationType::ACCESS => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                    } else {
                        selected_registry = selected_registries.0;
                    }

                    if specifics[1] {
                        let mut input = String::new();
                        if specifics[2] {
                            let _input = std::io::stdin().read_line(&mut input);
                            if input.len() == 1 && (input.chars().nth(0)).unwrap().is_ascii() {
                                selected_registry = ((input.chars().nth(0)).unwrap() as u8) as i32;
                            }
                        } else {
                            let _input = std::io::stdin().read_line(&mut input);
                            if input.len() == 1 && (input.chars().nth(0)).unwrap().is_numeric() {
                                let temp = input.parse::<i32>();
                                if temp.is_ok() {
                                    selected_registry = temp.unwrap();
                                }
                            }
                        }
                    } else {
                        if specifics[2] {
                            if selected_registry < 256 {
                                println!("{:?}", (selected_registry as u8) as char)
                            }
                        } else {
                            println!("{}", selected_registry);
                        }
                    }

                    if specifics[0] {
                        selected_registries.1 = selected_registry;
                    } else {
                        selected_registries.0 = selected_registry;
                    }

                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        polite_registries = selected_registries;
                    } else {
                        demanding_registries = selected_registries;
                    }
                },
                OperationType::LOOP => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                    } else {
                        selected_registry = selected_registries.0;
                    }

                    if specifics[2] {
                        if specifics[1] {
                            if loop_counters.1 < loop_registries.1 {
                                loop_counters.1 += 1;
                                i = loops.1 as usize;
                            }
                        } else {
                            loop_counters.1 = 0;
                            loop_registries.1 = selected_registry;
                            loops.1 = i as i32;
                        }
                    } else {
                        if specifics[1] {
                            if loop_counters.0 < loop_registries.0 {
                                loop_counters.0 += 1;
                                i = loops.0 as usize;
                            }
                        } else {
                            loop_counters.0 = 0;
                            loop_registries.0 = selected_registry;
                            loops.0 = i as i32;
                        }
                    }
                },
                OperationType::BRANCH_IF_GREATER => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    let mut other_registry = 0;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                        other_registry = selected_registries.0;
                    } else {
                        selected_registry = selected_registries.0;
                        other_registry = selected_registries.1;
                    }

                    if selected_registry > other_registry {
                        let mut value = 1;
                        if specifics[1] {
                            value += 2;
                        }
                        if specifics[2] {
                            value += 1;
                        }
                        if prefix_type == PrefixType::DEMANDING_STRONG && (social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= -1*MEDIUM_TOLERANCE_CLOSE) {
                            value *= 2;
                        }
                        if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                            i += value;
                        } else {
                            i -= value;
                        }
                    }
                }
                OperationType::BRANCH_IF_ZERO => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut selected_registry = 0;
                    if specifics[0] {
                        selected_registry = selected_registries.1;
                    } else {
                        selected_registry = selected_registries.0;
                    }

                    if selected_registry == 0 {
                        let mut value = 1;
                        if specifics[1] {
                            value += 2;
                        }
                        if specifics[2] {
                            value += 1;
                        }
                        if prefix_type == PrefixType::DEMANDING_STRONG && (social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= -1*MEDIUM_TOLERANCE_CLOSE) {
                            value *= 2;
                        }
                        if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                            i += value;
                        } else {
                            i -= value;
                        }
                    }
                }
                OperationType::BRANCH_IF_EQUAL => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    if selected_registries.0 == selected_registries.1 {
                        let mut value = 1;
                        if specifics[0] {
                            value += 4;
                        }
                        if specifics[1] {
                            value += 2;
                        }
                        if specifics[2] {
                            value += 1;
                        }
                        if prefix_type == PrefixType::DEMANDING_STRONG && (social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= -1*MEDIUM_TOLERANCE_CLOSE) {
                            value *= 2;
                        }
                        if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                            i += value;
                        } else {
                            i -= value;
                        }
                    }
                }
                OperationType::JUMP => {
                    let mut selected_registries: (i32, i32);
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        selected_registries = polite_registries;
                    } else {
                        selected_registries = demanding_registries;
                    }

                    let mut value = 1;
                    if specifics[0] {
                        value += 4;
                    }
                    if specifics[1] {
                        value += 2;
                    }
                    if specifics[2] {
                        value += 1;
                    }
                    if prefix_type == PrefixType::DEMANDING_STRONG && (social_credit >= MEDIUM_TOLERANCE_CLOSE || social_credit <= -1*MEDIUM_TOLERANCE_CLOSE) {
                        value *= 2;
                    }
                    if prefix_type == PrefixType::POLITE || prefix_type == PrefixType::POLITE_STRONG {
                        i += value;
                    } else {
                        i -= value;
                    }
                }
                _ => panic!(),
            }
        }
    }
}