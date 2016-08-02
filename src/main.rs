// The idea is to take a stream of bytes that looks like this:
//
// < One Byte Length > < Field A Data >
// < One Byte Length > < Field B Data >
// < One Byte Length > < Field C Data >
// < Two byte CRC16CCITT >
//
// Max length of fields:
//   A: 31 bytes, B: 40 bytes, C: 64 bytes
//   Total (incl. metadata): 140 bytes
//
// This data will be received asynchronously, and messages should be dispatched
//   as soon as they are available. You will not get all 140 characters unless
//
// Challenge mode (embedded-style):
//   1. No heap allocations
//   2. Minimize copies to reduce stack usage

fn main() {
    let input = vec![3, 1, 2, 3, 2, 1, 2, 4, 1, 2, 3, 4];

    let mut deser = Deserializer::new();

    println!("{:?}", deser.state);
    for i in input {
        println!("{:?} -> {:?}", i, deser.chew(i));
    }
    println!("{:?}", deser.state);
}

type FieldARaw = [u8; 31];
type FieldBRaw = [u8; 40];
type FieldCRaw = [u8; 64];

struct Field<T> {
    maxsize: Option<u8>,
    cursize: u8,
    rawdata: T,
}

#[derive(Debug)]
enum DeserState {
    WorkingA,
    WorkingB,
    WorkingC,
    Error,
    Complete,
}

struct Deserializer {
    field_a: Field<FieldARaw>,
    field_b: Field<FieldBRaw>,
    field_c: Field<FieldCRaw>,
    state: DeserState,
}

impl<T> Field<T> {

    // returns true if still working, false if done or error
    fn minichew(&mut self, b: u8) -> bool {
        if self.maxsize.is_none() {
            if b as usize <= self.rawdata.len() {
                self.maxsize = Some(b);
                return true;
            } else {
                // Notify an error some way?
                return false;
            }
        }

        self.rawdata[self.cursize as usize] = b;
        self.cursize += 1;

        self.cursize < self.maxsize.unwrap()
    }

    fn is_complete(&self) -> bool {
        match self.maxsize {
            Some(s) => s == self.cursize,
            None => false,
        }
    }
}

impl Deserializer {
    fn new() -> Deserializer {
        Deserializer {
            field_a: Field {
                maxsize: None,
                cursize: 0,
                rawdata: [0; 31],
            },
            field_b: Field {
                maxsize: None,
                cursize: 0,
                rawdata: [0; 40],
            },
            field_c: Field {
                maxsize: None,
                cursize: 0,
                rawdata: [0; 64],
            },
            state: DeserState::WorkingA,
        }
    }

    // returns true if still working, false if done or error
    fn chew(&mut self, b: u8) -> bool {
        match self.state {
            DeserState::WorkingA => {
                if self.field_a.minichew(b) && self.field_a.is_complete() {
                    self.state = DeserState::WorkingB;
                    true
                } else {
                    self.state = DeserState::Error;
                    false
                }
            },
            DeserState::WorkingB => {
                if self.field_b.minichew(b) && self.field_b.is_complete() {
                    self.state = DeserState::WorkingC;
                    true
                } else {
                    self.state = DeserState::Error;
                    false
                }
            },
            DeserState::WorkingC => {
                if self.field_c.minichew(b) && self.field_c.is_complete() {
                    self.state = DeserState::Complete;
                    true
                } else {
                    self.state = DeserState::Error;
                    false
                }
            },
            _ => false,
        }
    }
}