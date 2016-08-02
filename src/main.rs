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
                if self.field_a.maxsize.is_none() {
                    if b as usize <= self.field_a.rawdata.len() {
                        self.field_a.maxsize = Some(b);
                        return true;
                    } else {
                        self.state = DeserState::Error;
                        return false;
                    }
                }

                self.field_a.rawdata[self.field_a.cursize as usize] = b;
                self.field_a.cursize += 1;

                if self.field_a.cursize == self.field_a.maxsize.unwrap() {
                    self.state = DeserState::WorkingB;
                }

                true
            },
            DeserState::WorkingB => {
                if self.field_b.maxsize.is_none() {
                    if b as usize <= self.field_b.rawdata.len() {
                        self.field_b.maxsize = Some(b);
                        return true;
                    } else {
                        self.state = DeserState::Error;
                        return false;
                    }
                }

                self.field_b.rawdata[self.field_b.cursize as usize] = b;
                self.field_b.cursize += 1;

                if self.field_b.cursize == self.field_b.maxsize.unwrap() {
                    self.state = DeserState::WorkingC;
                }

                true
            },
            DeserState::WorkingC => {
                if self.field_c.maxsize.is_none() {
                    if b as usize <= self.field_c.rawdata.len() {
                        self.field_c.maxsize = Some(b);
                        return true;
                    } else {
                        self.state = DeserState::Error;
                        return false;
                    }
                }

                self.field_c.rawdata[self.field_c.cursize as usize] = b;
                self.field_c.cursize += 1;

                if self.field_c.cursize == self.field_c.maxsize.unwrap() {
                    self.state = DeserState::Complete;
                    return false;
                }

                true
            },
            _ => true,
        }
    }
}