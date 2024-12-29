use core::panic;

use itertools::Itertools;
use neerajsi::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Card {
    N(u8),
    T,
    J,
    Q,
    K,
    A,
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Card::N(v) => write!(f, "{}", *v+2),
            Card::T => write!(f, "T"),
            Card::J => write!(f, "J"),
            Card::Q => write!(f, "Q"),
            Card::K => write!(f, "K"),
            Card::A => write!(f, "A"),
        }
    }
}

fn map_card(card: u8) -> Card {
    match card {
        b'2'..=b'9' => Card::N(card - b'2'),
        b'T' => Card::T,
        b'J' => Card::J,
        b'Q' => Card::Q,
        b'K' => Card::K,
        b'A' => Card::A,
        _ => panic!("Unknown card {card}")
    }
}

const fn card_index(card: Card) -> usize {
    match card {
        Card::N(c) => c as usize,
        Card::T => 8,
        Card::J => 9,
        Card::Q => 10,
        Card::K => 11,
        Card::A => 12
    }
}

const CARD_COUNT: usize = 13;

fn counts_to_hand_type(counts: &[u8]) -> HandType {
    if counts.contains(&5) {
        HandType::FiveOfAKind
    } else if counts.contains(&4) {
        HandType::FourOfAKind
    } else if counts.contains(&3) && counts.contains(&2) {
        HandType::FullHouse
    } else if counts.contains(&3) {
        HandType::ThreeOfAKind
    } else if counts.iter().filter(|&c| *c == 2).count() == 2 {
        HandType::TwoPair
    } else if counts.contains(&2) {
        HandType::OnePair
    } else {
        HandType::HighCard
    }
}

fn main() {
    let input_raw = read_stdin_input();
    let input = std::str::from_utf8(&input_raw).unwrap();
    let lines = input.lines();

    let debug = std::env::args().nth(1).is_some();

    let hands = lines
        .map(|l| {
            let (cards, bid) = l.split_ascii_whitespace().collect_tuple().unwrap();
            let cards = cards.as_bytes();
            let bid: usize = bid.parse().unwrap();
            let cards = cards.iter().map(|&c| map_card(c)).collect_array::<5>().unwrap();

            let mut counts = [0u8; CARD_COUNT];
            cards.iter().for_each(|&c| counts[card_index(c)] += 1);
            let hand_type = counts_to_hand_type(&counts);
            
            let joker_index = card_index(Card::J);
            let jokers = counts[joker_index];
            counts[joker_index] = 0;

            let best_count = counts.iter().position_max().unwrap();
            counts[best_count] += jokers;

            let hand_type_jokers = counts_to_hand_type(&counts);
            let cards_with_jokers = cards.map(|c| {
                match c {
                    Card::J => 0u8,
                    _ => card_index(c) as u8 + 1
                }
            });

            (hand_type, cards, bid, hand_type_jokers, cards_with_jokers)
        })
        .collect_vec();
    
    let hands_sorted = hands.iter().sorted().collect_vec();

    let part1 = hands_sorted.iter().enumerate()
        .map(|(i, (h, cards, bid, ..))| {
            let rank = i + 1;
            if debug {
                println!("part1: {rank}: {h:?}, {} {bid}", cards.iter().format(""));
            }
            rank * bid
        }).sum::<usize>();
    
    dbg!(part1);

    let part2 = hands.iter().sorted_by_key(
        |(_ht, _cards, _bid, ht_jokers, cards_jokers)|
        (ht_jokers, cards_jokers)
    )
    .enumerate()
    .map(|(i, (_, cards, bid, ht, ..))| {
        let rank = i + 1;
        if debug {
            println!("part2: {rank}: {ht:?}, {} {bid}", cards.iter().format(""));
        }

        rank * bid
    })
    .sum::<usize>();

    dbg!(part2);
}
