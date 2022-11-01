use std::fmt;
use string_builder::Builder;

struct PReg{
    num: u32,
    ready: bool
}

impl PartialEq for PReg {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

impl ToString for PReg {
    fn to_string(&self) -> String {
        format!("PR#{} {}", self.num, if self.ready {"+"} else {" "}) 
    }
}

#[derive(PartialEq)]
enum VReg {
    F(u32),
    R(u32),
}

struct Inst {
    fu: u32,
    rt: VReg,
    rs1: VReg,
    rs2: VReg,
    delay: u32
}

struct ROBEntry {
    inst_ind: usize,
    rs_ind: usize,
    T: u32, 
    Told: u32,
    S: u32,
    X: u32,
    C: u32,
    R: u32,
}

impl ROBEntry {
    fn new(inst_ind: usize, rs_ind: usize, T: u32, Told: u32) -> Self {
        Self {
            inst_ind, rs_ind, T, Told,
            S: 0,
            X: 0,
            C: 0,
            R: 0
        }
    }
}

struct ROB {
    entries: Vec<ROBEntry>,
    max_entries: usize,
    head: usize,
    tail: Option<usize>
}

impl ROB {
    fn new(rob_size: usize) -> Self {
        let entries: Vec<ROBEntry> = Vec::with_capacity(rob_size);
        Self {
            entries: entries,
            max_entries: rob_size,
            head: 0,
            tail: None, // tail starts off in an uninitialized state
        }
    }
}

impl fmt::Display for ROB{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, entry) in self.entries.iter().enumerate() {
            //output += (if i == self.head.try_into().unwrap() { 'h' } else { ' ' }) + (if i == self.tail.try_into().unwrap() { 't' } else { ' ' }) + ' ';
            if i == self.head { write!(f, "h")?; } else { write!(f, " ")?; }
        
            // TODO: make else conditons all one?
            if let Some(head) = self.tail { 
                if head == i { write!(f, "t")?; } else { write!(f, " ")? }; 
            } else { write!(f, " ")? };

            write!(f, "  ")?;
            if entry.T    != 0 { write!(f, "PR#{} ", entry.T)?; } else { write!(f, "PR#   ")?; }
            if entry.Told != 0 { write!(f, "PR#{} ", entry.Told)?; } else { write!(f, "PR#   ")?; }
            if entry.S    != 0 { write!(f, "{} ", entry.S)?; } else { write!(f, "  ")?; }
            if entry.X    != 0 { write!(f, "{} ", entry.X)?; } else { write!(f, "  ")?; }
            if entry.C    != 0 { write!(f, "{} ", entry.C)?; } else { write!(f, "  ")?; }
            if entry.R    != 0 { write!(f, "{} ", entry.R)?; } else { write!(f, "  ")?; }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

struct ResStation_entry {
    fu_type: u32,
    fu_num: u32,
    rs_num: u32,
    busy: bool,
    T: u32,
    T1: PReg,
    T2: PReg,
}


fn main() {

    let mut my_rob : ROB = ROB::new(6);

    my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    my_rob.entries.push(ROBEntry::new(2,3, 7, 8));
    my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    my_rob.tail = Some(3);

    println!("{}", my_rob);
}

