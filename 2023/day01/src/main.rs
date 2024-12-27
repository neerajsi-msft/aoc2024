use neerajsi::{read_stdin_input, SumMultiple};

fn main() {
    let buf = read_stdin_input();

    let number_words = [
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine"
    ];

    let sum:[usize;2] = 
        buf.split(|&c| c == b'\n').map(|l| { 
            let (mut first, mut last) = (None, None);
            let (mut firstp2, mut lastp2) = (None, None);
            
            for (p, &c) in l.iter().enumerate() {
                match c {
                    b'0'..=b'9' => {
                        let c = (c - b'0') as usize;
                        first.get_or_insert(c);
                        last = Some(c);
                        firstp2.get_or_insert(c);
                        lastp2 = Some(c)
                    }
                    _ => {
                        let nw = number_words.iter().position(
                            |&nw| l[p..].starts_with(nw.as_bytes())
                        );

                        if let Some(n) = nw {
                            let n = n+1;
                            firstp2.get_or_insert(n);
                            lastp2 = Some(n)
    
                        }
                    }
                }
            }

            let conv_n = |a: Option<usize>, b: Option<usize>| {
                if a.is_some() {
                    a.unwrap() * 10 + b.unwrap()
                } else {
                    0
                }
            };

            [conv_n(first, last), conv_n(firstp2, lastp2)]
        })
        .sum_multiple();

    dbg!(sum);
}
